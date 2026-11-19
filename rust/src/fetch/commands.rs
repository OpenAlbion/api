//! The command implementations — the Rust port of
//! `app/Console/Commands/{Weapon,Armor,Accessory,Consumable,Migrations}`.
//!
//! Commands read the current JSON dataset, fetch wiki pages, merge the parsed
//! rows into the store and finally save (never deleting anything). A command
//! failing on one page only records a warning; the run keeps going.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};

use crate::fetch::aod::{self, AodItems};
use crate::fetch::parse::{self, ParseError, SpellDraft, SpellLinkDraft};
use crate::fetch::report::{backup_dataset, RunReport};
use crate::fetch::seeds;
use crate::fetch::store::{Store, TableId};
use crate::fetch::text::*;
use crate::fetch::transport::{fixture_name, wiki, FetchError, Transport};

/// Run-time options shared by every command.
#[derive(Debug, Clone)]
pub struct Options {
    pub data_dir: PathBuf,
    pub concurrency: usize,
    pub delay_ms: u64,
    pub limit: Option<usize>,
    pub dump_html: Option<PathBuf>,
    pub dry_run: bool,
    pub refresh_aod: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data"),
            concurrency: 3,
            delay_ms: 300,
            limit: None,
            dump_html: None,
            dry_run: false,
            refresh_aod: false,
        }
    }
}

/// HTML → spell links parser used by the spell commands.
type LinkParser = fn(&str, &str) -> Result<Vec<SpellLinkDraft>, ParseError>;

#[derive(Debug, Clone, Copy)]
pub enum SpellTarget {
    Weapon,
    Armor,
    Accessory,
    Mount,
}

pub struct Runner {
    pub store: Store,
    pub options: Options,
    transport: Arc<Transport>,
    report: RunReport,
    aod: Option<AodItems>,
}

impl Runner {
    pub fn new(options: Options, transport: Arc<Transport>, command: &str) -> io::Result<Self> {
        let store = Store::open(&options.data_dir)?;
        Ok(Self {
            store,
            options,
            transport,
            report: RunReport::new(command),
            aod: None,
        })
    }

    // -- infrastructure ---------------------------------------------------

    fn warn(&mut self, message: impl Into<String>) {
        self.report.warn(message);
    }

    fn dump_html(&self, url: &str, html: &str) {
        let Some(dir) = &self.options.dump_html else {
            return;
        };
        if fs::create_dir_all(dir).is_err() {
            return;
        }
        let path = dir.join(fixture_name(url));
        let _ = fs::write(path, html);
    }

    fn take_html(
        &mut self,
        url: &str,
        results: &HashMap<String, Result<String, FetchError>>,
    ) -> Option<String> {
        match results.get(url) {
            Some(Ok(html)) => {
                self.dump_html(url, html);
                Some(html.clone())
            }
            Some(Err(error)) => {
                self.warn(format!("{error}"));
                None
            }
            None => None,
        }
    }

    fn clamp(&self, count: usize) -> usize {
        match self.options.limit {
            Some(limit) => count.min(limit),
            None => count,
        }
    }

    /// Fetches all URLs concurrently (bounded by `concurrency`, spaced by
    /// `delay_ms`) and returns the result per URL.
    async fn fetch_many(&self, urls: &[String]) -> HashMap<String, Result<String, FetchError>> {
        let mut out = HashMap::new();
        if urls.is_empty() {
            return out;
        }
        let mut set = tokio::task::JoinSet::new();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.options.concurrency.max(1)));
        for url in urls {
            let transport = Arc::clone(&self.transport);
            let semaphore = Arc::clone(&semaphore);
            let url = url.clone();
            let delay = self.options.delay_ms;
            set.spawn(async move {
                let _permit = semaphore.acquire_owned().await.expect("semaphore");
                if delay > 0 {
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
                let result = transport.get_html(&url).await;
                (url, result)
            });
        }
        while let Some(joined) = set.join_next().await {
            if let Ok((url, result)) = joined {
                out.insert(url, result);
            }
        }
        out
    }

    async fn fetch_one(&mut self, url: &str) -> Option<String> {
        match self.transport.get_html(url).await {
            Ok(html) => {
                self.dump_html(url, &html);
                Some(html)
            }
            Err(error) => {
                self.warn(format!("{error}"));
                None
            }
        }
    }

    /// Loads the AOD item dump (downloaded/cached on first use).
    async fn ensure_aod(&mut self) -> bool {
        if self.aod.is_some() {
            return true;
        }
        let cache_path = self.options.data_dir.join(".cache").join("items.json");
        match aod::load_or_download(&cache_path, self.options.refresh_aod).await {
            Ok(items) if !items.is_empty() => {
                self.aod = Some(items);
                true
            }
            Ok(_) => {
                self.warn("AOD item dump appears empty — identifier fixups skipped");
                false
            }
            Err(error) => {
                self.warn(format!("AOD item dump unavailable: {error}"));
                false
            }
        }
    }

    fn parsed_or_warn<T>(&mut self, url: &str, result: Result<T, ParseError>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.warn(format!("cannot parse {url}: {error}"));
                None
            }
        }
    }

    // -- categories -------------------------------------------------------

    pub async fn weapon_category(&mut self) {
        // The weapon tree is only part of the category table.
        self.store.mark_partial(TableId::Categories);
        let url = wiki(seeds::WEAPON_CATEGORY_PAGE);
        let Some(html) = self.fetch_one(&url).await else {
            return;
        };
        let Some(categories) = self.parsed_or_warn(&url, parse::weapon_categories(&html, &url))
        else {
            return;
        };
        for category in categories {
            let parent = self
                .store
                .upsert_category(&category.primary, None, None, "weapon", 1);
            self.store.upsert_category(
                &category.secondary,
                Some(parent),
                category.path.as_deref(),
                "weapon",
                1,
            );
        }
    }

    pub fn armor_category(&mut self) {
        self.store.mark_partial(TableId::Categories);
        for (primary, secondary, path) in seeds::ARMOR_CATEGORIES {
            let parent = self.store.upsert_category(primary, None, None, "armor", 1);
            self.store
                .upsert_category(secondary, Some(parent), Some(path), "armor", 1);
        }
    }

    pub fn accessory_category(&mut self) {
        self.store.mark_partial(TableId::Categories);
        for (primary, secondary, path) in seeds::ACCESSORY_CATEGORIES {
            let parent = self
                .store
                .upsert_category(primary, None, Some(path), "accessory", 1);
            self.store
                .upsert_category(secondary, Some(parent), Some(path), "accessory", 1);
        }
    }

    pub fn consumable_category(&mut self) {
        self.store.mark_partial(TableId::Categories);
        for (primary, secondary, path) in seeds::CONSUMABLE_CATEGORIES {
            let parent = self
                .store
                .upsert_category(primary, None, *path, "consumable", 2);
            self.store
                .upsert_category(secondary, Some(parent), *path, "consumable", 2);
        }
    }

    pub async fn categories(&mut self) {
        self.weapon_category().await;
        self.armor_category();
        self.accessory_category();
        self.consumable_category();
    }

    // -- item lists -------------------------------------------------------

    pub async fn weapon(&mut self) {
        let subcategories = self.store.subcategories_with_path("weapon");
        for (subcategory_id, category_id, path) in subcategories {
            let url = wiki(&wiki_link(&path));
            let Some(html) = self.fetch_one(&url).await else {
                continue;
            };
            let Some(items) = self.parsed_or_warn(&url, parse::weapon_list(&html, &url)) else {
                continue;
            };
            for item in items {
                if item.name.is_empty() {
                    continue;
                }
                let identifier = item.icon.as_deref().and_then(identifier_from_icon);
                self.store.upsert_item(
                    TableId::Weapons,
                    &item.name,
                    Some(category_id),
                    Some(subcategory_id),
                    identifier.as_deref(),
                    item.path.as_deref(),
                    item.tier,
                    item.item_power,
                );
            }
        }
    }

    pub async fn armor(&mut self) {
        let subcategories = self.store.subcategories_with_path("armor");
        for (subcategory_id, category_id, path) in subcategories {
            let url = wiki(&wiki_link(&path));
            let Some(html) = self.fetch_one(&url).await else {
                continue;
            };
            let Some(items) = self.parsed_or_warn(&url, parse::armor_list(&html, &url)) else {
                continue;
            };
            for item in items {
                if item.name.is_empty() {
                    continue;
                }
                self.store.upsert_item(
                    TableId::Armors,
                    &item.name,
                    Some(category_id),
                    Some(subcategory_id),
                    None,
                    item.path.as_deref(),
                    item.tier,
                    item.item_power,
                );
            }
        }
    }

    pub async fn cape(&mut self) {
        self.store.mark_partial(TableId::Accessories);
        let url = wiki(seeds::CAPE_PAGE);
        let Some(html) = self.fetch_one(&url).await else {
            return;
        };
        let Some(items) = self.parsed_or_warn(&url, parse::cape_list(&html, &url)) else {
            return;
        };
        let Some(category_id) = self.store.category_by_name("Cape").map(|c| c.id) else {
            self.warn("Cape category missing — run `accessory-category` first");
            return;
        };
        for item in items {
            if item.name.is_empty() {
                continue;
            }
            self.store.upsert_item(
                TableId::Accessories,
                &item.name,
                Some(category_id),
                None,
                None,
                item.path.as_deref(),
                item.tier,
                item.item_power,
            );
        }
    }

    pub async fn bag(&mut self) {
        self.store.mark_partial(TableId::Accessories);
        let url = wiki(seeds::BAG_PAGE);
        let Some(html) = self.fetch_one(&url).await else {
            return;
        };
        let Some(items) = self.parsed_or_warn(&url, parse::bag_list(&html, &url)) else {
            return;
        };
        let Some(category_id) = self.store.category_by_name("Bag").map(|c| c.id) else {
            self.warn("Bag category missing — run `accessory-category` first");
            return;
        };
        let subcategory_id = self.store.category_by_name("Normal Bags").map(|c| c.id);
        for item in items {
            if item.name.is_empty() {
                continue;
            }
            self.store.upsert_item(
                TableId::Accessories,
                &item.name,
                Some(category_id),
                subcategory_id,
                None,
                item.path.as_deref(),
                item.tier,
                item.item_power,
            );
        }
    }

    pub async fn mount(&mut self) {
        self.store.mark_partial(TableId::Accessories);
        self.store.mark_partial(TableId::Categories);
        let url = wiki(seeds::MOUNT_PAGE);
        let Some(html) = self.fetch_one(&url).await else {
            return;
        };
        let Some(mounts) = self.parsed_or_warn(&url, parse::mount_list(&html, &url)) else {
            return;
        };
        let (name, path) = seeds::MOUNT_CATEGORY;
        let category_id = self
            .store
            .upsert_category(name, None, Some(path), "accessory", 1);
        for mount in mounts {
            if mount.name.is_empty() {
                continue;
            }
            let subcategory_name = mount.category.replace(':', "");
            let subcategory_id = self.store.upsert_category(
                &subcategory_name,
                Some(category_id),
                None,
                "accessory",
                1,
            );
            self.store.upsert_item(
                TableId::Accessories,
                &mount.name,
                Some(category_id),
                Some(subcategory_id),
                None,
                mount.path.as_deref(),
                None,
                None,
            );
        }
    }

    pub async fn food(&mut self) {
        // Potions are seeded separately, so foods are only part of the table.
        self.store.mark_partial(TableId::Consumables);
        let subcategories = self.store.subcategories_with_path("consumable");
        for (subcategory_id, category_id, path) in subcategories {
            let url = wiki(&wiki_link(&path));
            let Some(html) = self.fetch_one(&url).await else {
                continue;
            };
            let Some(items) = self.parsed_or_warn(&url, parse::food_list(&html, &url)) else {
                continue;
            };
            for item in items {
                if item.name.is_empty() {
                    continue;
                }
                self.store.upsert_consumable(
                    &item.name,
                    Some(category_id),
                    Some(subcategory_id),
                    None,
                    item.path.as_deref(),
                    item.tier,
                    item.item_power,
                );
            }
        }
    }

    pub fn potion(&mut self) {
        self.store.mark_partial(TableId::Consumables);
        for (name, tier, item_power, path, subcategory_name) in seeds::POTIONS {
            let Some(subcategory_id) = self.store.category_by_name(subcategory_name).map(|c| c.id)
            else {
                self.warn(format!(
                    "potion subcategory {subcategory_name:?} missing — run `consumable-category` first"
                ));
                continue;
            };
            let category_id = self
                .store
                .category_by_id(subcategory_id)
                .and_then(|sub| sub.parent_id);
            self.store.upsert_consumable(
                name,
                category_id,
                Some(subcategory_id),
                None,
                Some(path),
                parse_tier_text(tier),
                item_power_from_text(item_power),
            );
        }
    }

    // -- stats ------------------------------------------------------------

    async fn fetch_and_apply_stats(&mut self, kind: StatKind) {
        let (items, table) = match kind {
            StatKind::Weapon => (
                self.store.items_with_path(TableId::Weapons),
                TableId::WeaponStats,
            ),
            StatKind::Armor => (
                self.store.items_with_path(TableId::Armors),
                TableId::ArmorStats,
            ),
            StatKind::Accessory => (
                self.store.items_with_path(TableId::Accessories),
                TableId::AccessoryStats,
            ),
            StatKind::Consumable => (
                self.store.items_with_path(TableId::Consumables),
                TableId::ConsumableStats,
            ),
        };
        let mut items = items;
        if matches!(kind, StatKind::Weapon) {
            items.retain(|(_, path)| !seeds::WEAPON_STAT_SKIP_PATHS.contains(&path.as_str()));
        }
        if matches!(kind, StatKind::Accessory) {
            items.retain(|(_, path)| !seeds::ACCESSORY_STAT_SKIP_PATHS.contains(&path.as_str()));
        }
        let items: Vec<(i64, String)> = items.into_iter().take(self.clamp(usize::MAX)).collect();
        let urls: Vec<String> = items
            .iter()
            .map(|(_, path)| wiki(&wiki_link(path)))
            .collect();
        let results = self.fetch_many(&urls).await;

        for ((item_id, _path), url) in items.iter().zip(urls.iter()) {
            let Some(html) = self.take_html(url, &results) else {
                continue;
            };
            let parsed = match kind {
                StatKind::Weapon | StatKind::Armor => parse::weapon_or_armor_stats(&html, url),
                StatKind::Accessory => parse::accessory_stats(&html, url),
                StatKind::Consumable => parse::food_stats(&html, url),
            };
            let Some(drafts) = self.parsed_or_warn(url, parsed) else {
                continue;
            };
            for draft in drafts {
                self.apply_stat_draft(table, *item_id, &draft);
            }
        }

        // Potions use their own table position, and the Focus Restoration
        // Potion has no stats table at all — both are special-cased exactly
        // like the Laravel command.
        if matches!(kind, StatKind::Consumable) {
            let potions =
                self.consumable_items_in_category("Potions", Some(seeds::FOCUS_RESTORATION_POTION));
            let potions: Vec<(i64, String)> =
                potions.into_iter().take(self.clamp(usize::MAX)).collect();
            let urls: Vec<String> = potions
                .iter()
                .map(|(_, path)| wiki(&wiki_link(path)))
                .collect();
            let results = self.fetch_many(&urls).await;
            for ((item_id, _path), url) in potions.iter().zip(urls.iter()) {
                let Some(html) = self.take_html(url, &results) else {
                    continue;
                };
                let Some(drafts) = self.parsed_or_warn(url, parse::potion_stats(&html, url)) else {
                    continue;
                };
                for draft in drafts {
                    self.apply_stat_draft(table, *item_id, &draft);
                }
            }
            self.seed_focus_restoration_stats();
        }
    }

    fn apply_stat_draft(&mut self, table: TableId, item_id: i64, draft: &parse::StatDraft) {
        let Some(quality) = draft.quality.as_deref() else {
            return;
        };
        let Some(tier_text) = draft.tier.as_deref() else {
            return;
        };
        if quality.is_empty() || tier_text.is_empty() {
            return;
        }
        // A tier label that is not numeric ("Tier", empty) is a table label,
        // not a data column.
        if parse_tier_text(tier_text).is_none() {
            return;
        }
        let enchantment = enchantment_from_tier(tier_text);
        let stats: Vec<Value> = draft
            .stats
            .iter()
            .map(|(name, value)| json!({"name": name, "value": value}))
            .collect();
        self.store
            .upsert_stat(table, item_id, quality, enchantment, Value::Array(stats));
    }

    fn seed_focus_restoration_stats(&mut self) {
        let Some(consumable) = self
            .store
            .tables
            .consumables
            .iter()
            .find(|r| r.name == seeds::FOCUS_RESTORATION_POTION)
            .map(|r| r.id)
        else {
            self.warn("Focus Restoration Potion missing — seeded stat skipped");
            return;
        };
        let stats: Vec<Value> = seeds::focus_restoration_stats()
            .into_iter()
            .map(|(name, value)| json!({"name": name, "value": value}))
            .collect();
        self.store.upsert_stat(
            TableId::ConsumableStats,
            consumable,
            "Normal",
            0,
            Value::Array(stats),
        );
    }

    fn consumable_items_in_category(
        &self,
        category: &str,
        exclude_name: Option<&str>,
    ) -> Vec<(i64, String)> {
        let Some(category_id) = self.store.category_by_name(category).map(|c| c.id) else {
            return Vec::new();
        };
        self.store
            .tables
            .consumables
            .iter()
            .filter(|c| c.category_id == Some(category_id))
            .filter(|c| Some(c.name.as_str()) != exclude_name)
            .filter_map(|c| Some((c.id, c.path.clone()?)))
            .collect()
    }

    pub async fn weapon_stat(&mut self) {
        self.fetch_and_apply_stats(StatKind::Weapon).await;
    }

    pub async fn armor_stat(&mut self) {
        self.fetch_and_apply_stats(StatKind::Armor).await;
    }

    pub async fn accessory_stat(&mut self) {
        self.fetch_and_apply_stats(StatKind::Accessory).await;
    }

    pub async fn consumable_stat(&mut self) {
        self.fetch_and_apply_stats(StatKind::Consumable).await;
    }

    // -- spells -----------------------------------------------------------

    async fn link_and_upsert_spells(&mut self, target: SpellTarget) {
        // Each spell command only sees the spells of one item type.
        self.store.mark_partial(TableId::Spells);
        let (items, pivot_table, parser): (Vec<(i64, String)>, TableId, LinkParser) = match target {
            SpellTarget::Weapon => (
                self.store.items_with_path(TableId::Weapons),
                TableId::WeaponSpells,
                parse::weapon_spell_links,
            ),
            SpellTarget::Armor => (
                self.store.items_with_path(TableId::Armors),
                TableId::ArmorSpells,
                parse::armor_spell_links,
            ),
            SpellTarget::Accessory => (
                self.accessory_items(false),
                TableId::AccessorySpells,
                parse::accessory_spell_links,
            ),
            SpellTarget::Mount => (
                self.accessory_items(true),
                TableId::AccessorySpells,
                parse::mount_spell_links,
            ),
        };
        let items: Vec<(i64, String)> = items.into_iter().take(self.clamp(usize::MAX)).collect();

        let urls: Vec<String> = items
            .iter()
            .map(|(_, path)| wiki(&wiki_link(path)))
            .collect();
        let results = self.fetch_many(&urls).await;

        let mut links_per_item: Vec<(i64, Vec<SpellLinkDraft>)> = Vec::new();
        let mut spell_hrefs: BTreeSet<String> = BTreeSet::new();
        for ((item_id, _path), url) in items.iter().zip(urls.iter()) {
            let Some(html) = self.take_html(url, &results) else {
                continue;
            };
            let Some(links) = self.parsed_or_warn(url, parser(&html, url)) else {
                continue;
            };
            for link in &links {
                spell_hrefs.insert(link.href.clone());
            }
            links_per_item.push((*item_id, links));
        }

        // Fetch each unique spell page once.
        let spell_urls: Vec<String> = spell_hrefs.iter().map(|href| wiki(href)).collect();
        let spell_results = self.fetch_many(&spell_urls).await;
        let mut details: HashMap<String, SpellDraft> = HashMap::new();
        for href in &spell_hrefs {
            let url = wiki(href);
            let Some(html) = self.take_html(&url, &spell_results) else {
                continue;
            };
            if let Some(draft) = self.parsed_or_warn(&url, parse::spell_detail(&html, &url)) {
                details.insert(href.clone(), draft);
            }
        }

        for (item_id, links) in links_per_item {
            for link in links {
                let Some(draft) = details.get(&link.href) else {
                    continue;
                };
                let identifier = draft.icon.as_deref().and_then(identifier_from_icon);
                let description = draft.description.as_deref().map(sanitize_spell_description);
                let spell_id = self.store.upsert_spell(
                    &draft.name,
                    identifier.as_deref(),
                    &link.slot,
                    draft.attributes.clone(),
                    description.as_deref(),
                    draft.preview.as_deref(),
                    &wiki_link(&link.href),
                );
                self.store.link_spell(pivot_table, item_id, spell_id);
            }
        }
    }

    fn accessory_items(&self, mounts: bool) -> Vec<(i64, String)> {
        let mount_category = self.store.category_by_name("Mount").map(|c| c.id);
        self.store
            .tables
            .accessories
            .iter()
            .filter(|a| (a.category_id == mount_category) == mounts)
            .filter_map(|a| Some((a.id, a.path.clone()?)))
            .collect()
    }

    pub async fn weapon_spell(&mut self) {
        self.link_and_upsert_spells(SpellTarget::Weapon).await;
    }

    pub async fn armor_spell(&mut self) {
        self.link_and_upsert_spells(SpellTarget::Armor).await;
    }

    pub async fn accessory_spell(&mut self) {
        self.link_and_upsert_spells(SpellTarget::Accessory).await;
    }

    pub async fn mount_spell(&mut self) {
        self.link_and_upsert_spells(SpellTarget::Mount).await;
    }

    // -- consumable crafting & info ---------------------------------------

    pub async fn consumable_crafting(&mut self) {
        let _ = self.ensure_aod().await;
        let foods = self.consumable_items_in_category("Foods", None);
        let foods: Vec<(i64, String)> = foods.into_iter().take(self.clamp(usize::MAX)).collect();
        let urls: Vec<String> = foods
            .iter()
            .map(|(_, path)| wiki(&wiki_link(path)))
            .collect();
        let results = self.fetch_many(&urls).await;
        for ((item_id, _path), url) in foods.iter().zip(urls.iter()) {
            let Some(html) = self.take_html(url, &results) else {
                continue;
            };
            let Some(drafts) = self.parsed_or_warn(url, parse::food_crafting(&html, url)) else {
                continue;
            };
            self.apply_crafting(*item_id, drafts);
        }

        let potions =
            self.consumable_items_in_category("Potions", Some(seeds::FOCUS_RESTORATION_POTION));
        let potions: Vec<(i64, String)> =
            potions.into_iter().take(self.clamp(usize::MAX)).collect();
        let urls: Vec<String> = potions
            .iter()
            .map(|(_, path)| wiki(&wiki_link(path)))
            .collect();
        let results = self.fetch_many(&urls).await;
        for ((item_id, _path), url) in potions.iter().zip(urls.iter()) {
            let Some(html) = self.take_html(url, &results) else {
                continue;
            };
            let Some(drafts) = self.parsed_or_warn(url, parse::potion_crafting(&html, url)) else {
                continue;
            };
            self.apply_crafting(*item_id, drafts);
        }
    }

    fn apply_crafting(&mut self, consumable_id: i64, drafts: Vec<parse::CraftDraft>) {
        for draft in drafts {
            let Some(tier_text) = draft.tier.as_deref() else {
                continue;
            };
            if parse_tier_text(tier_text).is_none() {
                continue;
            }
            let enchantment = enchantment_from_tier(tier_text);
            let requirements = self.requirements_json(&draft.requirements);
            self.store
                .upsert_crafting(consumable_id, draft.per_craft, enchantment, requirements);
        }
    }

    fn requirements_json(&self, pairs: &[(String, String)]) -> Value {
        let mut out = Vec::new();
        for (name, value_text) in pairs {
            let value: i64 = value_text.trim().parse().unwrap_or(0);
            if value <= 0 {
                continue;
            }
            let identifier = self
                .aod
                .as_ref()
                .and_then(|items| items.unique_name(name))
                .map(str::to_string);
            let icon = crate::render::render_item(identifier.as_deref().unwrap_or(""));
            // Key order follows the SQLite jsonb export (alphabetical).
            out.push(json!({
                "icon": icon,
                "identifier": identifier,
                "name": name,
                "value": value,
            }));
        }
        Value::Array(out)
    }

    pub async fn consumable_info(&mut self) {
        let items = self.store.items_with_path(TableId::Consumables);
        let items: Vec<(i64, String)> = items.into_iter().take(self.clamp(usize::MAX)).collect();
        let urls: Vec<String> = items
            .iter()
            .map(|(_, path)| wiki(&wiki_link(path)))
            .collect();
        let results = self.fetch_many(&urls).await;
        for ((item_id, _path), url) in items.iter().zip(urls.iter()) {
            let Some(html) = self.take_html(url, &results) else {
                continue;
            };
            let Some(info) = self.parsed_or_warn(url, parse::consumable_info(&html, url)) else {
                continue;
            };
            self.store.set_consumable_info(*item_id, &info);
        }
    }

    // -- fixups -----------------------------------------------------------

    pub async fn identifiers(&mut self) {
        if !self.ensure_aod().await {
            return;
        }
        // Take the dump out of `self` so the store can be borrowed mutably.
        let aod = self.aod.take().expect("AOD dump loaded");

        let mut changed_weapons = false;
        for row in self
            .store
            .tables
            .weapons
            .iter_mut()
            .filter(|row| row.identifier.as_deref().unwrap_or("").is_empty())
        {
            let name = row.name.replace(" (Mount)", "");
            if let Some(unique) = aod.unique_name(&name) {
                row.identifier = Some(unique.split('@').next().unwrap_or(unique).to_string());
                changed_weapons = true;
            }
        }
        if changed_weapons {
            self.store.touch(TableId::Weapons);
        }

        let mut changed_armors = false;
        for row in self
            .store
            .tables
            .armors
            .iter_mut()
            .filter(|row| row.identifier.as_deref().unwrap_or("").is_empty())
        {
            if let Some(unique) = aod.unique_name(&row.name) {
                row.identifier = Some(unique.to_string());
                changed_armors = true;
            }
        }
        if changed_armors {
            self.store.touch(TableId::Armors);
        }

        let mut changed_accessories = false;
        for row in self
            .store
            .tables
            .accessories
            .iter_mut()
            .filter(|row| row.identifier.as_deref().unwrap_or("").is_empty())
        {
            let name = row.name.replace(" (Mount)", "");
            if let Some(unique) = aod.unique_name(&name) {
                row.identifier = Some(unique.to_string());
                changed_accessories = true;
            }
        }
        if changed_accessories {
            self.store.touch(TableId::Accessories);
        }

        let mut changed_consumables = false;
        for row in self
            .store
            .tables
            .consumables
            .iter_mut()
            .filter(|row| row.identifier.as_deref().unwrap_or("").is_empty())
        {
            if let Some(unique) = aod.unique_name(&row.name) {
                row.identifier = Some(unique.to_string());
                changed_consumables = true;
            }
        }
        if changed_consumables {
            self.store.touch(TableId::Consumables);
        }
        self.aod = Some(aod);
    }

    pub fn item_power(&mut self) {
        self.fill_item_power(TableId::Weapons, false);
        self.fill_item_power(TableId::Armors, false);
        self.fill_item_power(TableId::Accessories, true);
    }

    fn fill_item_power(&mut self, table: TableId, zero_means_none: bool) {
        let stats_table = match table {
            TableId::Weapons => TableId::WeaponStats,
            TableId::Armors => TableId::ArmorStats,
            TableId::Accessories => TableId::AccessoryStats,
            _ => return,
        };
        let ids: Vec<i64> = match table {
            TableId::Weapons => self
                .store
                .tables
                .weapons
                .iter()
                .filter(|r| r.item_power.is_none())
                .map(|r| r.id)
                .collect(),
            TableId::Armors => self
                .store
                .tables
                .armors
                .iter()
                .filter(|r| r.item_power.is_none())
                .map(|r| r.id)
                .collect(),
            TableId::Accessories => self
                .store
                .tables
                .accessories
                .iter()
                .filter(|r| r.item_power.is_none())
                .map(|r| r.id)
                .collect(),
            _ => Vec::new(),
        };
        if ids.is_empty() {
            return;
        }
        let mut changed = false;
        for id in ids {
            let value = self.item_power_stat_value(stats_table, id);
            let Some(value) = value else { continue };
            let parsed = value.parse::<i64>().ok();
            let new_power = if zero_means_none {
                parsed.filter(|v| *v != 0)
            } else {
                parsed
            };
            if let Some(power) = new_power {
                let row = match table {
                    TableId::Weapons => self.store.tables.weapons.iter_mut().find(|r| r.id == id),
                    TableId::Armors => self.store.tables.armors.iter_mut().find(|r| r.id == id),
                    TableId::Accessories => self
                        .store
                        .tables
                        .accessories
                        .iter_mut()
                        .find(|r| r.id == id),
                    _ => None,
                };
                if let Some(row) = row {
                    row.item_power = Some(power);
                    changed = true;
                }
            }
        }
        if changed {
            self.store.touch(table);
        }
    }

    fn item_power_stat_value(&self, table: TableId, item_id: i64) -> Option<String> {
        match table {
            TableId::WeaponStats => self
                .store
                .tables
                .weapon_stats
                .iter()
                .filter(|r| r.item_id == item_id && r.quality == "Normal" && r.enchantment == 0)
                .find_map(|r| stat_entry(&r.stats, "Item Power")),
            TableId::ArmorStats => self
                .store
                .tables
                .armor_stats
                .iter()
                .filter(|r| r.item_id == item_id && r.quality == "Normal" && r.enchantment == 0)
                .find_map(|r| stat_entry(&r.stats, "Item Power")),
            TableId::AccessoryStats => self
                .store
                .tables
                .accessory_stats
                .iter()
                .filter(|r| r.item_id == item_id && r.quality == "Normal" && r.enchantment == 0)
                .find_map(|r| stat_entry(&r.stats, "Item Power")),
            _ => None,
        }
    }

    pub fn mount_tier(&mut self) {
        let Some(mount_category) = self.store.category_by_name("Mount").map(|c| c.id) else {
            return;
        };
        let mut changed = false;
        for row in self
            .store
            .tables
            .accessories
            .iter_mut()
            .filter(|r| r.category_id == Some(mount_category) && r.tier.is_none())
        {
            let Some(identifier) = row.identifier.as_deref() else {
                continue;
            };
            let tier = (2..=8).find(|tier| identifier.starts_with(&format!("T{tier}_")));
            if let Some(tier) = tier {
                row.tier = Some(tier as f64);
                changed = true;
            }
        }
        if changed {
            self.store.touch(TableId::Accessories);
        }
    }

    pub fn cape_subcategory(&mut self) {
        let (Some(cape), Some(normal), Some(faction)) = (
            self.store.category_by_name("Cape").map(|c| c.id),
            self.store.category_by_name("Normal Capes").map(|c| c.id),
            self.store.category_by_name("Faction Capes").map(|c| c.id),
        ) else {
            return;
        };
        let with_spells: BTreeSet<i64> = self
            .store
            .tables
            .accessory_spells
            .iter()
            .map(|r| r.item_id)
            .collect();
        let mut changed = false;
        for row in self
            .store
            .tables
            .accessories
            .iter_mut()
            .filter(|r| r.category_id == Some(cape))
        {
            let target = if with_spells.contains(&row.id) {
                faction
            } else {
                normal
            };
            if row.subcategory_id != Some(target) {
                row.subcategory_id = Some(target);
                changed = true;
            }
        }
        if changed {
            self.store.touch(TableId::Accessories);
        }
    }

    pub fn bag_subcategory(&mut self) {
        let (Some(bag), Some(normal)) = (
            self.store.category_by_name("Bag").map(|c| c.id),
            self.store.category_by_name("Normal Bags").map(|c| c.id),
        ) else {
            return;
        };
        let mut changed = false;
        for row in self
            .store
            .tables
            .accessories
            .iter_mut()
            .filter(|r| r.category_id == Some(bag))
        {
            if row.subcategory_id != Some(normal) {
                row.subcategory_id = Some(normal);
                changed = true;
            }
        }
        if changed {
            self.store.touch(TableId::Accessories);
        }
    }

    pub fn sanitize_spell_description(&mut self) {
        let mut changed = false;
        for row in self.store.tables.spells.iter_mut() {
            if let Some(description) = &row.description {
                let sanitized = sanitize_spell_description(description);
                if sanitized != *description {
                    row.description = Some(sanitized);
                    changed = true;
                }
            }
        }
        if changed {
            self.store.touch(TableId::Spells);
        }
    }

    pub async fn update_spell_description(&mut self) {
        let targets: Vec<(i64, String)> = self
            .store
            .tables
            .spells
            .iter()
            .filter(|r| r.description.as_deref() == Some("Sanitize Spell Description"))
            .map(|r| (r.id, r.reference.clone()))
            .collect();
        let urls: Vec<String> = targets
            .iter()
            .map(|(_, path)| wiki(&wiki_link(path)))
            .collect();
        let results = self.fetch_many(&urls).await;
        let mut changed = false;
        for ((id, _path), url) in targets.iter().zip(urls.iter()) {
            let Some(html) = self.take_html(url, &results) else {
                continue;
            };
            let Some(draft) = self.parsed_or_warn(url, parse::spell_detail(&html, url)) else {
                continue;
            };
            let Some(description) = draft.description else {
                continue;
            };
            if let Some(row) = self.store.tables.spells.iter_mut().find(|r| r.id == *id) {
                row.description = Some(sanitize_spell_description(&description));
                changed = true;
            }
        }
        if changed {
            self.store.touch(TableId::Spells);
        }
    }

    // -- full pipeline ----------------------------------------------------

    pub async fn all(&mut self) {
        self.weapon_category().await;
        self.armor_category();
        self.accessory_category();
        self.consumable_category();
        self.weapon().await;
        self.armor().await;
        self.cape().await;
        self.bag().await;
        self.mount().await;
        self.food().await;
        self.potion();
        self.identifiers().await;
        self.weapon_stat().await;
        self.armor_stat().await;
        self.accessory_stat().await;
        self.consumable_stat().await;
        self.weapon_spell().await;
        self.armor_spell().await;
        self.accessory_spell().await;
        self.mount_spell().await;
        self.sanitize_spell_description();
        self.consumable_crafting().await;
        self.consumable_info().await;
        self.item_power();
        self.mount_tier();
        self.cape_subcategory();
        self.bag_subcategory();
    }

    // -- finishing --------------------------------------------------------

    /// Applies the degenerate-fetch guard, prints the diff and saves.
    /// Returns the number of warnings + errors (0 = clean run).
    pub fn finish(mut self, command: &str) -> io::Result<usize> {
        // Never write a table whose fetch produced zero rows while the old
        // data is non-empty (a broken parser must not shrink a table).
        let mut guarded = Vec::new();
        for table in self.store.touched_tables() {
            if self.store.fetched_any(table)
                && self.store.fetched_count(table) == 0
                && self.store.old.len(table) > 0
            {
                guarded.push(table);
            }
        }
        for table in guarded {
            self.warn(format!(
                "{}: this fetch produced no rows but {} exist — file left untouched",
                table.file(),
                self.store.old.len(table)
            ));
            self.store.unmark_table(table);
        }

        let mut diff = RunReport::compute(
            command,
            &self.store.old,
            self.store.fetched_keys(),
            self.store.partial_tables(),
        );
        diff.warnings = std::mem::take(&mut self.report.warnings);
        diff.errors = std::mem::take(&mut self.report.errors);
        let problems = diff.warnings.len() + diff.errors.len();
        diff.print();

        if self.options.dry_run {
            println!("dry run — nothing written");
            return Ok(problems);
        }
        if !self.store.has_changes() {
            println!("no changes to save");
            return Ok(problems);
        }

        let backup = backup_dataset(&self.options.data_dir)?;
        self.store.save()?;
        if let Some(dir) = backup {
            let report_path = diff.write_json(&dir)?;
            println!("backup + report: {}", report_path.display());
        }
        Ok(problems)
    }
}

/// Reads one named entry out of a `[{name, value}]` stats array.
fn stat_entry(stats: &Value, name: &str) -> Option<String> {
    let Value::Array(entries) = stats else {
        return None;
    };
    entries
        .iter()
        .find(|entry| entry.get("name").and_then(Value::as_str) == Some(name))
        .and_then(|entry| entry.get("value").and_then(Value::as_str))
        .map(str::to_string)
}

#[derive(Debug, Clone, Copy)]
enum StatKind {
    Weapon,
    Armor,
    Accessory,
    Consumable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::store::Tables;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "openalbion-cmd-{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn fixture_transport() -> Arc<Transport> {
        Arc::new(Transport::Fixture(
            crate::fetch::transport::FixtureFetcher::new(PathBuf::from("/nonexistent")),
        ))
    }

    #[tokio::test]
    async fn category_seeds_create_the_tree_offline() {
        let dir = temp_dir("seeds");
        let options = Options {
            data_dir: dir.clone(),
            ..Options::default()
        };
        let mut runner = Runner::new(options, fixture_transport(), "test").unwrap();
        runner.armor_category();
        runner.accessory_category();
        runner.consumable_category();
        runner.potion();
        runner.finish("test").unwrap();

        let tables = Tables::load(&dir).unwrap();
        // Armor: 3 primaries + 9 secondaries.
        assert_eq!(
            tables
                .categories
                .iter()
                .filter(|c| c.item_type == "armor")
                .count(),
            12
        );
        // Accessory: Cape, Bag primaries + Normal Capes, Faction Capes, Normal Bags.
        assert_eq!(
            tables
                .categories
                .iter()
                .filter(|c| c.item_type == "accessory")
                .count(),
            5
        );
        // Consumables: Foods + 7 subs, Potions + 8 subs.
        assert_eq!(
            tables
                .categories
                .iter()
                .filter(|c| c.item_type == "consumable")
                .count(),
            17
        );
        // Potions: 20 seeded rows.
        assert_eq!(tables.consumables.len(), 20);
        assert!(tables
            .consumables
            .iter()
            .any(|c| c.name == "Focus Restoration Potion"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn finish_never_writes_a_degenerate_table() {
        let dir = temp_dir("guard");
        let options = Options {
            data_dir: dir.clone(),
            ..Options::default()
        };
        let mut runner = Runner::new(options, fixture_transport(), "test").unwrap();
        runner.store.upsert_item(
            TableId::Weapons,
            "Old Sword",
            None,
            None,
            None,
            None,
            Some(4.0),
            None,
        );
        runner.store.save().unwrap();
        // A fetch run that produced zero weapon rows (broken parser / blocked
        // page): the existing file must stay untouched.
        runner.store = Store::open(&dir).unwrap();
        runner.store.note_fetch_attempt(TableId::Weapons);
        let problems = runner.finish("test").unwrap();
        assert!(problems > 0, "the guard must surface a warning");
        let reloaded = Tables::load(&dir).unwrap();
        assert_eq!(
            reloaded.weapons.len(),
            1,
            "degenerate fetch must not wipe the table"
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
