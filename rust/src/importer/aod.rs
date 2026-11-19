//! Loading `ao-bin-dumps`: the public game-data dump used as the authoritative
//! item source.
//!
//! Two files are needed, both plain GitHub raw downloads (no crawling):
//! - `items.json`            — game item definitions (tier, power, categories,
//!                             crafting requirements, enchantment variants);
//! - `formatted/items.json`  — English names and descriptions.
//!
//! Only the sections holding equippable/consumable gameplay items are parsed;
//! the rest of the dump (world, mobs, quests, …) is ignored.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

pub const ITEMS_URL: &str =
    "https://raw.githubusercontent.com/ao-data/ao-bin-dumps/master/items.json";
pub const FORMATTED_ITEMS_URL: &str =
    "https://raw.githubusercontent.com/ao-data/ao-bin-dumps/master/formatted/items.json";

/// Sections of `items.json` that contain gameplay items we map.
const SECTIONS: &[&str] = &[
    "weapon",
    "transformationweapon",
    "equipmentitem",
    "consumableitem",
    "mount",
];

/// One crafting resource line (`craftresource`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pub unique_name: String,
    pub count: i64,
}

/// A crafting recipe for one enchantment level.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Recipe {
    /// `@amountcrafted` — how many items one craft produces.
    pub amount_crafted: Option<i64>,
    pub resources: Vec<Resource>,
}

/// One item definition from the dump, normalized.
#[derive(Debug, Clone)]
pub struct AodItem {
    pub unique_name: String,
    /// English display name (`LocalizedNames.EN-US`).
    pub name: String,
    /// English description (`LocalizedDescriptions.EN-US`).
    pub description: Option<String>,
    pub tier: Option<i64>,
    pub item_power: Option<i64>,
    pub slot_type: Option<String>,
    pub shop_category: Option<String>,
    pub shop_subcategory1: Option<String>,
    pub shop_subcategory2: Option<String>,
    pub mount_category: Option<String>,
    /// Recipe for enchantment 0 and each enchantment variant (index = level).
    pub recipes: Vec<Recipe>,
}

impl AodItem {
    /// True for entries that are not real gameplay items (vanity, test/GM
    /// builds, event and non-tradable duplicates).
    pub fn is_cosmetic_or_test(&self) -> bool {
        const ID_MARKERS: &[&str] = &[
            "_VANITY_",
            "VANITY",
            "_TEST",
            "TEST_",
            "DEBUG",
            "GAMEMASTER",
            "RUBBERBAND",
            "NONTRADABLE",
            "EVENTITEM",
            // cosmetics and promotional sets
            "_FOUNDER_",
            "STARTERPACK",
            "TELLAFRIEND",
            "_XMAS",
            "EVENT_",
            "ARENA_BANNER",
            "DRAKE_EGG",
        ];
        const NAME_MARKERS: &[&str] = &["game master", "vanity", "test ", "decorative "];
        let upper = self.unique_name.to_uppercase();
        if ID_MARKERS.iter().any(|marker| upper.contains(marker)) {
            return true;
        }
        if matches!(
            self.shop_category.as_deref(),
            Some("vanity") | Some("event")
        ) {
            return true;
        }
        let lower = self.name.to_lowercase();
        NAME_MARKERS.iter().any(|marker| lower.contains(marker))
    }
}

/// The whole dump, indexed by unique name.
#[derive(Debug, Default)]
pub struct AodDump {
    pub items: HashMap<String, AodItem>,
}

impl AodDump {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get(&self, unique_name: &str) -> Option<&AodItem> {
        self.items.get(unique_name)
    }

    /// All items sorted by unique name (deterministic output).
    pub fn sorted(&self) -> Vec<&AodItem> {
        let mut items: Vec<&AodItem> = self.items.values().collect();
        items.sort_by(|a, b| a.unique_name.cmp(&b.unique_name));
        items
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

fn as_str(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn as_i64(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(|v| {
        v.as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| v.as_i64())
    })
}

fn entries(value: &Value) -> Vec<&Value> {
    match value {
        Value::Array(rows) => rows.iter().collect(),
        Value::Object(_) => vec![value],
        _ => Vec::new(),
    }
}

/// `craftresource` may be a single object or an array.
fn parse_resources(value: &Value) -> Vec<Resource> {
    entries(value)
        .into_iter()
        .filter_map(|entry| {
            let unique_name = as_str(entry, "@uniquename")?;
            let count = as_i64(entry, "@count").unwrap_or(0);
            Some(Resource { unique_name, count })
        })
        .collect()
}

fn parse_recipe(value: &Value) -> Option<Recipe> {
    if !value.is_object() {
        return None;
    }
    let resources = value
        .get("craftresource")
        .map(parse_resources)
        .unwrap_or_default();
    let amount_crafted = as_i64(value, "@amountcrafted");
    if resources.is_empty() && amount_crafted.is_none() {
        return None;
    }
    Some(Recipe {
        amount_crafted,
        resources,
    })
}

/// Builds the recipe list: index 0 = base recipe, then enchantment levels.
fn parse_recipes(entry: &Value) -> Vec<Recipe> {
    let mut recipes = vec![
        parse_recipe(entry.get("craftingrequirements").unwrap_or(&Value::Null)).unwrap_or_default(),
    ];
    let variants = entry
        .get("enchantments")
        .and_then(|e| e.get("enchantment"))
        .map(entries)
        .unwrap_or_default();
    let mut by_level: Vec<(i64, Recipe)> = Vec::new();
    for variant in variants {
        let Some(level) = as_i64(variant, "@enchantmentlevel") else {
            continue;
        };
        if let Some(recipe) =
            parse_recipe(variant.get("craftingrequirements").unwrap_or(&Value::Null))
        {
            by_level.push((level, recipe));
        }
    }
    by_level.sort_by_key(|(level, _)| *level);
    for (level, recipe) in by_level {
        let index = level.max(0) as usize;
        while recipes.len() <= index {
            recipes.push(Recipe::default());
        }
        recipes[index] = recipe;
    }
    recipes
}

/// Parses `items.json` + `formatted/items.json` into a normalized dump.
pub fn parse(items_json: &str, formatted_json: &str) -> io::Result<AodDump> {
    let root: Value = serde_json::from_str(items_json).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid items.json: {e}"),
        )
    })?;
    let formatted: Value = serde_json::from_str(formatted_json).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid formatted/items.json: {e}"),
        )
    })?;

    // unique name -> (English name, English description)
    let mut names: HashMap<&str, (&str, Option<&str>)> = HashMap::new();
    for entry in entries(&formatted) {
        let Some(unique_name) = entry.get("UniqueName").and_then(Value::as_str) else {
            continue;
        };
        let name = entry
            .get("LocalizedNames")
            .and_then(|n| n.get("EN-US"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let description = entry
            .get("LocalizedDescriptions")
            .and_then(|d| d.get("EN-US"))
            .and_then(Value::as_str);
        names.insert(unique_name, (name, description));
    }

    let items_root = root.get("items").unwrap_or(&Value::Null);
    let mut dump = AodDump::default();
    for section in SECTIONS {
        let Some(value) = items_root.get(*section) else {
            continue;
        };
        for entry in entries(value) {
            let Some(unique_name) = as_str(entry, "@uniquename") else {
                continue;
            };
            let (name, description) = names
                .get(unique_name.as_str())
                .map(|(n, d)| (*n, *d))
                .unwrap_or(("", None));
            let item = AodItem {
                unique_name: unique_name.clone(),
                name: name.trim().to_string(),
                description: description
                    .map(|d| d.trim().to_string())
                    .filter(|d| !d.is_empty()),
                tier: as_i64(entry, "@tier"),
                // Gear uses @itempower, consumables @dummyitempower.
                item_power: as_i64(entry, "@itempower")
                    .or_else(|| as_i64(entry, "@dummyitempower")),
                slot_type: as_str(entry, "@slottype"),
                shop_category: as_str(entry, "@shopcategory"),
                shop_subcategory1: as_str(entry, "@shopsubcategory1"),
                shop_subcategory2: as_str(entry, "@shopsubcategory2"),
                mount_category: as_str(entry, "@mountcategory"),
                recipes: parse_recipes(entry),
            };
            dump.items.insert(unique_name, item);
        }
    }
    Ok(dump)
}

// ---------------------------------------------------------------------------
// Downloading / caching
// ---------------------------------------------------------------------------

pub fn cache_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(".cache").join("ao-bin-dumps")
}

/// Loads the dump from `source`: either a local directory (a checkout/download
/// of the repo) or the GitHub raw URLs, caching downloads under the data dir.
pub async fn load(data_dir: &Path, source: Option<&Path>, refresh: bool) -> io::Result<AodDump> {
    let (items_text, formatted_text) = match source {
        Some(dir) => {
            let items = fs::read_to_string(dir.join("items.json")).map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!("cannot read {}: {e}", dir.join("items.json").display()),
                )
            })?;
            let formatted = fs::read_to_string(dir.join("formatted/items.json")).map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!(
                        "cannot read {}: {e}",
                        dir.join("formatted/items.json").display()
                    ),
                )
            })?;
            (items, formatted)
        }
        None => {
            let dir = cache_dir(data_dir);
            fs::create_dir_all(&dir)?;
            let items_path = dir.join("items.json");
            let formatted_path = dir.join("formatted-items.json");
            let items = fetch_cached(&items_path, ITEMS_URL, refresh).await?;
            let formatted = fetch_cached(&formatted_path, FORMATTED_ITEMS_URL, refresh).await?;
            (items, formatted)
        }
    };
    parse(&items_text, &formatted_text)
}

async fn fetch_cached(path: &Path, url: &str, refresh: bool) -> io::Result<String> {
    if !refresh && path.exists() {
        if let Ok(text) = fs::read_to_string(path) {
            if !text.is_empty() {
                return Ok(text);
            }
        }
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(io::Error::other)?;
    let response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "openalbion-import")
        .send()
        .await
        .map_err(|e| io::Error::other(format!("downloading {url}: {e}")))?;
    if !response.status().is_success() {
        return Err(io::Error::other(format!(
            "downloading {url}: HTTP {}",
            response.status()
        )));
    }
    let text = response.text().await.map_err(io::Error::other)?;
    fs::write(path, &text)?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEMS: &str = r#"{
      "items": {
        "weapon": [{
          "@uniquename": "T4_MAIN_AXE", "@tier": "4", "@itempower": "700",
          "@slottype": "mainhand", "@shopcategory": "weapons",
          "@shopsubcategory1": "axe", "@shopsubcategory2": "axe_main_axe",
          "craftingrequirements": {
            "@amountcrafted": "1",
            "craftresource": [{"@uniquename": "T4_PLANKS", "@count": "8"}, {"@uniquename": "T4_METALBAR", "@count": "16"}]
          },
          "enchantments": {"enchantment": [{
            "@enchantmentlevel": "1",
            "craftingrequirements": {"@amountcrafted": "1", "craftresource": [{"@uniquename": "T4_PLANKS", "@count": "8"}, {"@uniquename": "T4_METALBAR", "@count": "16"}, {"@uniquename": "T4_RUNE", "@count": "8"}]}
          }]}
        }],
        "transformationweapon": [{
          "@uniquename": "T4_2H_SHAPESHIFTER_SET1", "@tier": "4", "@itempower": "700",
          "@slottype": "mainhand", "@shopcategory": "weapons", "@shopsubcategory1": "shapeshifterstaff"
        }],
        "consumableitem": [{
          "@uniquename": "T4_POTION_HEAL", "@tier": "4", "@dummyitempower": "700",
          "@slottype": "potion", "@shopcategory": "consumables",
          "@shopsubcategory1": "potions", "@shopsubcategory2": "heal",
          "craftingrequirements": {"@amountcrafted": "5", "craftresource": {"@uniquename": "T4_BURDOCK", "@count": "24"}}
        }],
        "mount": [{
          "@uniquename": "T5_MOUNT_ARMORED_HORSE", "@tier": "5", "@itempower": "800",
          "@slottype": "mount", "@shopcategory": "mounts", "@mountcategory": "HORSE",
          "@shopsubcategory1": "basemounts", "@shopsubcategory2": "armoredhorse"
        }],
        "simpleitem": [{"@uniquename": "T1_WOOD"}]
      }
    }"#;

    const FORMATTED: &str = r#"[
      {"UniqueName": "T4_MAIN_AXE", "LocalizedNames": {"EN-US": "Adept's Battleaxe"},
       "LocalizedDescriptions": {"EN-US": "A battleaxe."}},
      {"UniqueName": "T4_2H_SHAPESHIFTER_SET1", "LocalizedNames": {"EN-US": "Adept's Prowling Staff"}},
      {"UniqueName": "T4_POTION_HEAL", "LocalizedNames": {"EN-US": "Healing Potion"},
       "LocalizedDescriptions": {"EN-US": "Restores health over time."}},
      {"UniqueName": "T5_MOUNT_ARMORED_HORSE", "LocalizedNames": {"EN-US": "Expert's Armored Horse"}}
    ]"#;

    #[test]
    fn parses_sections_with_names_and_recipes() {
        let dump = parse(ITEMS, FORMATTED).unwrap();
        // simpleitem is not a gameplay section we map.
        assert_eq!(dump.len(), 4);

        let axe = dump.get("T4_MAIN_AXE").unwrap();
        assert_eq!(axe.name, "Adept's Battleaxe");
        assert_eq!(axe.description.as_deref(), Some("A battleaxe."));
        assert_eq!(axe.tier, Some(4));
        assert_eq!(axe.item_power, Some(700));
        assert_eq!(axe.slot_type.as_deref(), Some("mainhand"));
        assert_eq!(axe.shop_subcategory1.as_deref(), Some("axe"));
        // base recipe + enchantment 1
        assert_eq!(axe.recipes.len(), 2);
        assert_eq!(axe.recipes[0].resources.len(), 2);
        assert_eq!(axe.recipes[1].resources.len(), 3);
        assert_eq!(axe.recipes[1].resources[2].unique_name, "T4_RUNE");

        // transformationweapon section is included.
        assert!(dump.get("T4_2H_SHAPESHIFTER_SET1").is_some());
        // single-object craftresource is normalized to a list.
        let potion = dump.get("T4_POTION_HEAL").unwrap();
        assert_eq!(potion.recipes[0].amount_crafted, Some(5));
        assert_eq!(potion.recipes[0].resources.len(), 1);
        // name-only entries still parse (no description).
        let mount = dump.get("T5_MOUNT_ARMORED_HORSE").unwrap();
        assert_eq!(mount.mount_category.as_deref(), Some("HORSE"));
        assert_eq!(mount.description, None);
    }

    #[test]
    fn flags_cosmetics_and_test_items() {
        let make = |id: &str, name: &str, shop: Option<&str>| AodItem {
            unique_name: id.to_string(),
            name: name.to_string(),
            description: None,
            tier: None,
            item_power: None,
            slot_type: None,
            shop_category: shop.map(str::to_string),
            shop_subcategory1: None,
            shop_subcategory2: None,
            mount_category: None,
            recipes: Vec::new(),
        };
        assert!(
            make("UNIQUE_HEAD_VANITY_WARRIOR_KING", "Paragon's Crown", None).is_cosmetic_or_test()
        );
        assert!(make("T4_SWORD_TEST", "Sword", None).is_cosmetic_or_test());
        assert!(
            make("UNIQUE_INTERNAL_HEAD_GAMEMASTER", "Game Master's Hat", None)
                .is_cosmetic_or_test()
        );
        assert!(make("T4_FIREWORK", "Firework", Some("vanity")).is_cosmetic_or_test());
        assert!(!make("T4_MAIN_AXE", "Adept's Battleaxe", Some("weapons")).is_cosmetic_or_test());
        // Mists "Calming Potion" is a real item, not a test entry.
        assert!(
            !make("T5_POTION_MOB_RESET", "Calming Potion", Some("consumables"))
                .is_cosmetic_or_test()
        );
    }

    #[test]
    fn recipes_fill_enchantment_levels() {
        let bump = |level: u32| {
            format!(
                r#"{{"@uniquename":"T4_X","@tier":"4","enchantments":{{"enchantment":[
                  {{"@enchantmentlevel":"{level}","craftingrequirements":{{"@amountcrafted":"1","craftresource":[{{"@uniquename":"T4_RUNE","@count":"8"}}]}}}}
                ]}}}}"#
            )
        };
        let items = format!(r#"{{"items":{{"weapon":[{}]}}}}"#, bump(3));
        let dump = parse(
            &items,
            r#"[{"UniqueName":"T4_X","LocalizedNames":{"EN-US":"X"}}]"#,
        )
        .unwrap();
        let item = dump.get("T4_X").unwrap();
        // levels 1 and 2 are empty placeholders, level 3 carries the recipe.
        assert_eq!(item.recipes.len(), 4);
        assert!(item.recipes[1].resources.is_empty());
        assert_eq!(item.recipes[3].resources[0].count, 8);
    }
}
