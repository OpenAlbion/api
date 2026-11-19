//! Application state: the whole dataset loaded from JSON files into memory,
//! plus the small runtime pieces (rate limiter, AOD proxy cache, config).

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use dashmap::DashMap;
use serde_json::Value;

use crate::models::{Category, Consumable, CraftingRow, Item, Spell, SpellPivot, StatRow};

/// Fixed-window rate limiter state (shared across all `/api/*` routes,
/// mirroring Laravel's named `api` limiter: 60 requests per minute per IP).
#[derive(Debug)]
pub struct RateLimitState {
    pub limit: u64,
    window: Mutex<(i64, DashMap<String, u64>)>, // (window minute, ip -> hits)
}

impl RateLimitState {
    pub fn new(limit: u64) -> Self {
        Self { limit, window: Mutex::new((0, DashMap::new())) }
    }

    /// Records a hit for `ip` in the current 60-second window.
    /// Returns `Some(retry_after_seconds)` when the request is over limit.
    pub fn hit(&self, ip: &str, now_unix: i64) -> Option<u64> {
        let current_window = now_unix.div_euclid(60);
        let mut guard = self.window.lock().expect("rate limit state poisoned");
        if guard.0 != current_window {
            guard.0 = current_window;
            guard.1.clear();
        }
        let hits = {
            let mut entry = guard.1.entry(ip.to_string()).or_insert(0);
            *entry.value_mut() += 1;
            *entry.value()
        };
        if hits > self.limit {
            let seconds_left = (current_window + 1) * 60 - now_unix;
            Some(seconds_left.max(1) as u64)
        } else {
            None
        }
    }
}

/// TTL cache for the Albion Online Data price proxy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AodCacheKey {
    pub region: String,
    pub item_id: String,
    /// Sorted query pairs, excluding `api_token` (mirrors the Laravel
    /// `generateCacheKey` behavior of dropping it from the cache key).
    pub query: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct AodCache {
    ttl: Duration,
    /// key -> (inserted at, unix millis; value)
    entries: Mutex<HashMap<AodCacheKey, (u64, Value)>>,
}

impl AodCache {
    pub fn new(ttl: Duration) -> Self {
        Self { ttl, entries: Mutex::new(HashMap::new()) }
    }

    fn fresh(&self, at: u64, now: u64) -> bool {
        now.saturating_sub(at) < self.ttl.as_millis() as u64
    }

    pub fn get(&self, key: &AodCacheKey) -> Option<Value> {
        let now = crate::platform::now_millis();
        let mut guard = self.entries.lock().expect("aod cache poisoned");
        match guard.get(key) {
            Some((at, value)) if self.fresh(*at, now) => Some(value.clone()),
            Some(_) => {
                guard.remove(key);
                None
            }
            None => None,
        }
    }

    pub fn put(&self, key: AodCacheKey, value: Value) {
        let now = crate::platform::now_millis();
        let mut guard = self.entries.lock().expect("aod cache poisoned");
        // Opportunistic sweep of expired entries so the map cannot grow
        // without bound.
        guard.retain(|_, (at, _)| now.saturating_sub(*at) < self.ttl.as_millis() as u64);
        guard.insert(key, (now, value));
    }
}

/// Data tables that must be present in `DATA_DIR`.
const REQUIRED_TABLES: &[&str] = &[
    "categories",
    "weapons",
    "armors",
    "accessories",
    "consumables",
    "spells",
    "weapon_spells",
    "armor_spells",
    "accessory_spells",
    "weapon_stats",
    "armor_stats",
    "accessory_stats",
    "consumable_stats",
    "consumable_craftings",
];

/// Runtime configuration (environment in production, tweaks in tests).
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub slack_webhook_url: Option<String>,
    pub rate_limit_per_minute: u64,
    pub aod_cache_ttl: Duration,
    /// AOD price-proxy URL template; `{region}` and `{item}` are replaced.
    pub aod_url_template: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            slack_webhook_url: None,
            rate_limit_per_minute: 60,
            aod_cache_ttl: Duration::from_secs(180),
            aod_url_template:
                "http://{region}.albion-online-data.com/api/v2/stats/prices/{item}.json".to_string(),
        }
    }
}

pub struct AppState {
    pub categories: Vec<Category>,
    pub weapons: Vec<Item>,
    pub armors: Vec<Item>,
    pub accessories: Vec<Item>,
    pub consumables: Vec<Consumable>,
    pub spells: Vec<Spell>,

    /// category id -> index into `categories`
    pub category_index: HashMap<i64, usize>,
    /// item id -> index into the respective Vec
    pub weapon_index: HashMap<i64, usize>,
    pub armor_index: HashMap<i64, usize>,
    pub accessory_index: HashMap<i64, usize>,
    pub consumable_index: HashMap<i64, usize>,

    /// item id -> set of spell ids (result order comes from the spells table)
    pub weapon_spells: HashMap<i64, HashSet<i64>>,
    pub armor_spells: HashMap<i64, HashSet<i64>>,
    pub accessory_spells: HashMap<i64, HashSet<i64>>,

    /// item id -> stat rows in table (rowid) order
    pub weapon_stats: HashMap<i64, Vec<StatRow>>,
    pub armor_stats: HashMap<i64, Vec<StatRow>>,
    pub accessory_stats: HashMap<i64, Vec<StatRow>>,
    pub consumable_stats: HashMap<i64, Vec<StatRow>>,
    /// consumable id -> crafting rows in table (rowid) order
    pub consumable_craftings: HashMap<i64, Vec<CraftingRow>>,

    pub slack_webhook_url: Option<String>,
    pub aod_url_template: String,
    /// Where bug reports are appended (server builds with a data dir only).
    pub bug_reports_path: Option<PathBuf>,
    pub rate_limit: RateLimitState,
    pub aod_cache: AodCache,
    #[cfg(feature = "server")]
    pub http_client: reqwest::Client,
}

/// Returns the raw JSON bytes of one data table (e.g. `"weapons"`).
pub type TableLoader<'a> = dyn Fn(&str) -> std::io::Result<Cow<'static, [u8]>> + 'a;

fn read_table_file(data_dir: &Path, table: &str) -> std::io::Result<Cow<'static, [u8]>> {
    let path = data_dir.join(format!("{table}.json"));
    std::fs::read(&path).map(Cow::Owned).map_err(|e| {
        std::io::Error::new(
            e.kind(),
            format!(
                "cannot open {path}: {e} — run `python3 scripts/export_sqlite.py` to generate the data files",
                path = path.display()
            ),
        )
    })
}

fn load_vec<T: for<'de> serde::Deserialize<'de>>(loader: &TableLoader<'_>, table: &str) -> std::io::Result<Vec<T>> {
    let bytes = loader(table)?;
    serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::other(format!("invalid JSON in {table}.json: {e}")))
}

fn group_by_item<T>(rows: Vec<T>, key: impl Fn(&T) -> i64) -> HashMap<i64, Vec<T>> {
    let mut map: HashMap<i64, Vec<T>> = HashMap::new();
    for row in rows {
        map.entry(key(&row)).or_default().push(row);
    }
    map
}

fn index_by_id<T>(rows: &[T], id: impl Fn(&T) -> i64) -> HashMap<i64, usize> {
    rows.iter().enumerate().map(|(i, r)| (id(r), i)).collect()
}

impl AppState {
    /// Loads every data file from `data_dir` and builds the lookup indices.
    pub fn load(data_dir: &Path, config: AppConfig) -> std::io::Result<Self> {
        let mut state = Self::from_loader(&|table| read_table_file(data_dir, table), config)?;
        state.bug_reports_path = Some(data_dir.join("bug-reports.jsonl"));
        Ok(state)
    }

    /// Builds the state from tables supplied by `loader` (files on the
    /// server, bytes embedded in the binary on Workers).
    pub fn from_loader(loader: &TableLoader<'_>, config: AppConfig) -> std::io::Result<Self> {
        let categories: Vec<Category> = load_vec(loader, "categories")?;
        let weapons: Vec<Item> = load_vec(loader, "weapons")?;
        let armors: Vec<Item> = load_vec(loader, "armors")?;
        let accessories: Vec<Item> = load_vec(loader, "accessories")?;
        let consumables: Vec<Consumable> = load_vec(loader, "consumables")?;
        let spells: Vec<Spell> = load_vec(loader, "spells")?;

        let weapon_spells: Vec<SpellPivot> = load_vec(loader, "weapon_spells")?;
        let armor_spells: Vec<SpellPivot> = load_vec(loader, "armor_spells")?;
        let accessory_spells: Vec<SpellPivot> = load_vec(loader, "accessory_spells")?;
        let weapon_spells = group_by_item(weapon_spells, |r| r.item_id)
            .into_iter()
            .map(|(k, rows)| (k, rows.into_iter().map(|r| r.spell_id).collect::<HashSet<_>>()))
            .collect::<HashMap<_, _>>();
        let armor_spells = group_by_item(armor_spells, |r| r.item_id)
            .into_iter()
            .map(|(k, rows)| (k, rows.into_iter().map(|r| r.spell_id).collect::<HashSet<_>>()))
            .collect::<HashMap<_, _>>();
        let accessory_spells = group_by_item(accessory_spells, |r| r.item_id)
            .into_iter()
            .map(|(k, rows)| (k, rows.into_iter().map(|r| r.spell_id).collect::<HashSet<_>>()))
            .collect::<HashMap<_, _>>();

        let weapon_stats: Vec<StatRow> = load_vec(loader, "weapon_stats")?;
        let armor_stats: Vec<StatRow> = load_vec(loader, "armor_stats")?;
        let accessory_stats: Vec<StatRow> = load_vec(loader, "accessory_stats")?;
        let consumable_stats: Vec<StatRow> = load_vec(loader, "consumable_stats")?;
        let weapon_stats = group_by_item(weapon_stats, |r| r.item_id);
        let armor_stats = group_by_item(armor_stats, |r| r.item_id);
        let accessory_stats = group_by_item(accessory_stats, |r| r.item_id);
        let consumable_stats = group_by_item(consumable_stats, |r| r.item_id);

        let craftings: Vec<CraftingRow> = load_vec(loader, "consumable_craftings")?;
        let consumable_craftings = group_by_item(craftings, |r| r.item_id);

        let category_index = index_by_id(&categories, |c| c.id);
        let weapon_index = index_by_id(&weapons, |w| w.id);
        let armor_index = index_by_id(&armors, |a| a.id);
        let accessory_index = index_by_id(&accessories, |a| a.id);
        let consumable_index = index_by_id(&consumables, |c| c.id);

        tracing::info!(
            categories = categories.len(),
            weapons = weapons.len(),
            armors = armors.len(),
            accessories = accessories.len(),
            consumables = consumables.len(),
            spells = spells.len(),
            "dataset loaded"
        );

        #[cfg(feature = "server")]
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|e| std::io::Error::other(format!("http client: {e}")))?;

        Ok(Self {
            categories,
            weapons,
            armors,
            accessories,
            consumables,
            spells,
            category_index,
            weapon_index,
            armor_index,
            accessory_index,
            consumable_index,
            weapon_spells,
            armor_spells,
            accessory_spells,
            weapon_stats,
            armor_stats,
            accessory_stats,
            consumable_stats,
            consumable_craftings,
            slack_webhook_url: config.slack_webhook_url,
            aod_url_template: config.aod_url_template,
            bug_reports_path: None,
            rate_limit: RateLimitState::new(config.rate_limit_per_minute),
            aod_cache: AodCache::new(config.aod_cache_ttl),
            #[cfg(feature = "server")]
            http_client,
        })
    }

    pub fn category(&self, id: Option<i64>) -> Option<&Category> {
        id.and_then(|id| self.category_index.get(&id)).map(|&i| &self.categories[i])
    }
}

/// Sanity check at startup that all required files were consumed.
pub fn check_data_dir(data_dir: &Path) -> std::io::Result<()> {
    for table in REQUIRED_TABLES {
        let path = data_dir.join(format!("{table}.json"));
        if !path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "missing {path} — run `python3 scripts/export_sqlite.py` to generate the data files",
                    path = path.display()
                ),
            ));
        }
    }
    Ok(())
}
