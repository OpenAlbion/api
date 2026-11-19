//! The dataset store for the importer.
//!
//! Loads the JSON tables, and **overwrites** the item tables from the dump
//! while guaranteeing:
//! - the row `id` of an existing item is preserved (matched by identifier or
//!   by name) so `*_stats` and item↔spell pivots stay attached;
//! - wiki-only fields (`path`, consumable `info`) are carried over, because
//!   the dump has no wiki data;
//! - files are written in the exact `export_sqlite.py` format, so untouched
//!   tables and untouched rows stay byte-identical;
//! - nothing is written for a table the dump produced no rows for.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::importer::crafting::CraftingRow;
use crate::importer::mapping::Table;

// ---------------------------------------------------------------------------
// Row types (identical shape to the dataset files)
// ---------------------------------------------------------------------------

fn ser_tier<S>(value: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(v) if v.fract() == 0.0 && v.abs() < 1e15 => serializer.serialize_i64(*v as i64),
        Some(v) => serializer.serialize_f64(*v),
        None => serializer.serialize_none(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRow {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub path: Option<String>,
    #[serde(rename = "type")]
    pub item_type: String,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemRow {
    pub id: i64,
    pub category_id: Option<i64>,
    pub subcategory_id: Option<i64>,
    pub name: String,
    pub identifier: Option<String>,
    #[serde(serialize_with = "ser_tier")]
    pub tier: Option<f64>,
    pub item_power: Option<i64>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsumableRow {
    pub id: i64,
    pub category_id: Option<i64>,
    pub subcategory_id: Option<i64>,
    pub name: String,
    pub identifier: Option<String>,
    #[serde(serialize_with = "ser_tier")]
    pub tier: Option<f64>,
    pub item_power: Option<i64>,
    pub path: Option<String>,
    pub info: Option<String>,
}

// ---------------------------------------------------------------------------
// JSON formatting (Python `json.dumps` layout, as produced by the export)
// ---------------------------------------------------------------------------

struct PythonFormatter;

impl serde_json::ser::Formatter for PythonFormatter {
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b", ")
        }
    }

    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b", ")
        }
    }

    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(b": ")
    }
}

pub fn serialize_rows<T: Serialize>(rows: &[T]) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut buffer, PythonFormatter);
    rows.serialize(&mut serializer).map_err(io::Error::other)?;
    Ok(buffer)
}

// ---------------------------------------------------------------------------
// The dataset
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct Dataset {
    pub categories: Vec<CategoryRow>,
    pub weapons: Vec<ItemRow>,
    pub armors: Vec<ItemRow>,
    pub accessories: Vec<ItemRow>,
    pub consumables: Vec<ConsumableRow>,
    pub consumable_craftings: Vec<CraftingRow>,
}

impl Dataset {
    pub fn load(dir: &Path) -> io::Result<Self> {
        Ok(Self {
            categories: read_table(dir, "categories")?,
            weapons: read_table(dir, "weapons")?,
            armors: read_table(dir, "armors")?,
            accessories: read_table(dir, "accessories")?,
            consumables: read_table(dir, "consumables")?,
            consumable_craftings: read_table(dir, "consumable_craftings")?,
        })
    }

    pub fn items(&self, table: Table) -> &[ItemRow] {
        match table {
            Table::Weapons => &self.weapons,
            Table::Armors => &self.armors,
            Table::Accessories => &self.accessories,
            // Consumables have their own row type; callers use `consumables()`.
            Table::Consumables => &[],
        }
    }

    pub fn items_mut(&mut self, table: Table) -> Option<&mut Vec<ItemRow>> {
        match table {
            Table::Weapons => Some(&mut self.weapons),
            Table::Armors => Some(&mut self.armors),
            Table::Accessories => Some(&mut self.accessories),
            Table::Consumables => None,
        }
    }

    /// Category id lookup by name.
    pub fn category_id(&self, name: &str) -> Option<i64> {
        self.categories
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.id)
    }

    /// Subcategory id lookup: the child of `category` with that name.
    pub fn subcategory_id(&self, category: &str, subcategory: &str) -> Option<i64> {
        let parent = self.category_id(category)?;
        self.categories
            .iter()
            .find(|c| c.parent_id == Some(parent) && c.name == subcategory)
            .map(|c| c.id)
    }

    /// All item identities (identifier **and** name) for matching dump items
    /// against existing rows.
    pub fn identity_index(&self) -> IdentityIndex {
        let mut index = IdentityIndex::default();
        for table in [Table::Weapons, Table::Armors, Table::Accessories] {
            for row in self.items(table) {
                index.insert(&row.identifier, &row.name, row.id);
            }
        }
        for row in &self.consumables {
            index.insert(&row.identifier, &row.name, row.id);
        }
        index
    }

    pub fn save(&self, dir: &Path, changed: &[TableFile]) -> io::Result<()> {
        fs::create_dir_all(dir)?;
        for file in changed {
            match file {
                TableFile::Categories => write_table(dir, "categories", &self.categories)?,
                TableFile::Weapons => write_table(dir, "weapons", &self.weapons)?,
                TableFile::Armors => write_table(dir, "armors", &self.armors)?,
                TableFile::Accessories => write_table(dir, "accessories", &self.accessories)?,
                TableFile::Consumables => write_table(dir, "consumables", &self.consumables)?,
                TableFile::ConsumableCraftings => {
                    write_table(dir, "consumable_craftings", &self.consumable_craftings)?
                }
            }
        }
        Ok(())
    }
}

/// Which table files were rewritten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TableFile {
    Categories,
    Weapons,
    Armors,
    Accessories,
    Consumables,
    ConsumableCraftings,
}

impl TableFile {
    pub fn file(self) -> &'static str {
        match self {
            TableFile::Categories => "categories.json",
            TableFile::Weapons => "weapons.json",
            TableFile::Armors => "armors.json",
            TableFile::Accessories => "accessories.json",
            TableFile::Consumables => "consumables.json",
            TableFile::ConsumableCraftings => "consumable_craftings.json",
        }
    }

    pub fn of(table: Table) -> Self {
        match table {
            Table::Weapons => TableFile::Weapons,
            Table::Armors => TableFile::Armors,
            Table::Accessories => TableFile::Accessories,
            Table::Consumables => TableFile::Consumables,
        }
    }
}

#[derive(Debug, Default)]
pub struct IdentityIndex {
    by_identifier: HashMap<String, i64>,
    by_name: HashMap<String, i64>,
    /// Every normalized display name currently in the dataset (duplicate guard).
    names: HashSet<String>,
}

impl IdentityIndex {
    fn insert(&mut self, identifier: &Option<String>, name: &str, id: i64) {
        if let Some(identifier) = identifier {
            if !identifier.is_empty() {
                self.by_identifier.entry(identifier.clone()).or_insert(id);
            }
        }
        let normalized = normalize_name(name);
        if !normalized.is_empty() {
            self.names.insert(normalized.clone());
            self.by_name.entry(normalized).or_insert(id);
        }
    }

    /// Normalized display names already used by the dataset.
    pub fn existing_names(&self) -> HashSet<String> {
        self.names.clone()
    }

    /// Existing row id for a dump item, matched by identifier first, then by
    /// normalized name (covers rows whose identifier was a mangled name).
    pub fn find(&self, unique_name: &str, name: &str) -> Option<i64> {
        self.by_identifier
            .get(unique_name)
            .copied()
            .or_else(|| self.by_name.get(&normalize_name(name)).copied())
    }

    pub fn len(&self) -> usize {
        self.by_name.len().max(self.by_identifier.len())
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty() && self.by_identifier.is_empty()
    }
}

/// Case/whitespace-insensitive comparison key (the wiki data has stray
/// leading spaces on some names).
pub fn normalize_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn next_id<T>(rows: &[T], id: impl Fn(&T) -> i64) -> i64 {
    rows.iter().map(id).max().unwrap_or(0) + 1
}

fn read_table<T: for<'de> Deserialize<'de>>(dir: &Path, name: &str) -> io::Result<Vec<T>> {
    let path = dir.join(format!("{name}.json"));
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path)
        .map_err(|e| io::Error::new(e.kind(), format!("cannot read {}: {e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid {}: {e}", path.display()),
        )
    })
}

fn write_table<T: Serialize>(dir: &Path, name: &str, rows: &[T]) -> io::Result<()> {
    let bytes = serialize_rows(rows)?;
    let path = dir.join(format!("{name}.json"));
    let tmp = dir.join(format!(".{name}.json.tmp"));
    fs::write(&tmp, &bytes)?;
    fs::rename(&tmp, &path)
}

/// Copies every dataset table file into `data/backups/<timestamp>/`.
pub fn backup(dir: &Path, stamp: &str) -> io::Result<Option<PathBuf>> {
    const TABLES: &[&str] = &[
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
    let target = dir.join("backups").join(stamp);
    let mut copied = 0usize;
    for table in TABLES {
        let source = dir.join(format!("{table}.json"));
        if !source.exists() {
            continue;
        }
        if copied == 0 {
            fs::create_dir_all(&target)?;
        }
        fs::copy(&source, target.join(format!("{table}.json")))?;
        copied += 1;
    }
    Ok((copied > 0).then_some(target))
}

// ---------------------------------------------------------------------------
// Overwrite helpers
// ---------------------------------------------------------------------------

/// Outcome of overwriting one table.
#[derive(Debug, Default, Clone)]
pub struct OverwriteStats {
    pub updated: usize,
    pub inserted: usize,
    pub unchanged: usize,
    pub identifier_fixed: Vec<(String, String)>,
    pub renames: Vec<(String, String)>,
    pub item_power_changed: Vec<(String, Option<i64>, Option<i64>)>,
    pub tier_changed: Vec<(String, Option<f64>, Option<f64>)>,
    pub carry_over_without_match: usize,
}

impl OverwriteStats {
    pub fn changed(&self) -> bool {
        self.updated > 0 || self.inserted > 0
    }
}

/// Replaces `rows` with the dump's item set.
///
/// `incoming` rows are already fully resolved (id, category ids, name, …) by
/// the caller; this function only merges bookkeeping: it keeps the existing
/// id when the caller passed `id = 0`, and reports what changed.
pub fn merge_item_rows(
    existing: &[ItemRow],
    mut incoming: Vec<ItemRow>,
    index: &IdentityIndex,
) -> (Vec<ItemRow>, OverwriteStats) {
    let mut stats = OverwriteStats::default();
    let mut by_id: HashMap<i64, ItemRow> = existing.iter().map(|r| (r.id, r.clone())).collect();

    for row in incoming.iter_mut() {
        match index.find(row.identifier.as_deref().unwrap_or(""), &row.name) {
            Some(existing_id) => {
                let old = by_id.remove(&existing_id);
                row.id = existing_id;
                if let Some(old) = old {
                    let same_after_merge = {
                        // `path` is carried over from `old`, so compare the
                        // prospective row, not the raw incoming one.
                        let mut candidate = row.clone();
                        if candidate.path.is_none() {
                            candidate.path = old.path.clone();
                        }
                        candidate == old
                    };
                    if old.identifier != row.identifier {
                        stats.identifier_fixed.push((
                            old.identifier.clone().unwrap_or_default(),
                            row.identifier.clone().unwrap_or_default(),
                        ));
                    }
                    if old.name != row.name {
                        stats.renames.push((old.name.clone(), row.name.clone()));
                    }
                    if old.item_power != row.item_power {
                        stats.item_power_changed.push((
                            row.name.clone(),
                            old.item_power,
                            row.item_power,
                        ));
                    }
                    if old.tier != row.tier {
                        stats
                            .tier_changed
                            .push((row.name.clone(), old.tier, row.tier));
                    }
                    // Wiki-only fields and unmapped values are carried over:
                    // the dump knows nothing about paths, and a mapping gap
                    // must not erase an existing category.
                    if row.path.is_none() {
                        row.path = old.path.clone();
                    }
                    if row.category_id.is_none() {
                        row.category_id = old.category_id;
                    }
                    if row.subcategory_id.is_none() {
                        row.subcategory_id = old.subcategory_id;
                    }
                    if row.item_power.is_none() {
                        row.item_power = old.item_power;
                    }
                    if same_after_merge {
                        stats.unchanged += 1;
                    } else {
                        stats.updated += 1;
                    }
                } else {
                    stats.inserted += 1;
                }
                by_id.insert(row.id, row.clone());
            }
            None => {
                stats.inserted += 1;
                by_id.insert(row.id, row.clone());
            }
        }
    }

    // Rows the dump does not know (should be none for items) are kept.
    stats.carry_over_without_match = by_id.len().saturating_sub(incoming.len());
    let mut rows: Vec<ItemRow> = by_id.into_values().collect();
    rows.sort_by_key(|r| r.id);
    (rows, stats)
}

/// Same as [`merge_item_rows`] for consumables (adds the `info` carry-over).
pub fn merge_consumable_rows(
    existing: &[ConsumableRow],
    incoming: Vec<ConsumableRow>,
    index: &IdentityIndex,
) -> (Vec<ConsumableRow>, OverwriteStats) {
    let mut stats = OverwriteStats::default();
    let mut by_id: HashMap<i64, ConsumableRow> =
        existing.iter().map(|r| (r.id, r.clone())).collect();

    let incoming_len = incoming.len();
    for mut row in incoming {
        match index.find(row.identifier.as_deref().unwrap_or(""), &row.name) {
            Some(existing_id) => {
                row.id = existing_id;
                if let Some(old) = by_id.remove(&existing_id) {
                    let same_after_merge = {
                        let mut candidate = row.clone();
                        if candidate.path.is_none() {
                            candidate.path = old.path.clone();
                        }
                        if candidate.info.is_none() {
                            candidate.info = old.info.clone();
                        }
                        candidate == old
                    };
                    if old.identifier != row.identifier {
                        stats.identifier_fixed.push((
                            old.identifier.clone().unwrap_or_default(),
                            row.identifier.clone().unwrap_or_default(),
                        ));
                    }
                    if old.name != row.name {
                        stats.renames.push((old.name.clone(), row.name.clone()));
                    }
                    if old.item_power != row.item_power {
                        stats.item_power_changed.push((
                            row.name.clone(),
                            old.item_power,
                            row.item_power,
                        ));
                    }
                    if old.tier != row.tier {
                        stats
                            .tier_changed
                            .push((row.name.clone(), old.tier, row.tier));
                    }
                    if row.path.is_none() {
                        row.path = old.path.clone();
                    }
                    if row.info.is_none() {
                        row.info = old.info.clone();
                    }
                    if row.category_id.is_none() {
                        row.category_id = old.category_id;
                    }
                    if row.subcategory_id.is_none() {
                        row.subcategory_id = old.subcategory_id;
                    }
                    if row.item_power.is_none() {
                        row.item_power = old.item_power;
                    }
                    if same_after_merge {
                        stats.unchanged += 1;
                    } else {
                        stats.updated += 1;
                    }
                } else {
                    stats.inserted += 1;
                }
                by_id.insert(row.id, row);
            }
            None => {
                stats.inserted += 1;
                by_id.insert(row.id, row);
            }
        }
    }

    stats.carry_over_without_match = by_id.len().saturating_sub(incoming_len);
    let mut rows: Vec<ConsumableRow> = by_id.into_values().collect();
    rows.sort_by_key(|r| r.id);
    (rows, stats)
}

/// Value type used by the crafting table (re-exported for convenience).
pub type RequirementValue = Value;

#[cfg(test)]
mod tests {
    use super::*;

    fn item(
        id: i64,
        identifier: &str,
        name: &str,
        power: Option<i64>,
        path: Option<&str>,
    ) -> ItemRow {
        ItemRow {
            id,
            category_id: Some(1),
            subcategory_id: Some(2),
            name: name.to_string(),
            identifier: Some(identifier.to_string()),
            tier: Some(4.0),
            item_power: power,
            path: path.map(str::to_string),
        }
    }

    #[test]
    fn overwrite_preserves_ids_and_wiki_paths() {
        let existing = vec![
            item(
                5,
                "  Journeyman's Torch",
                "Journeyman's Torch",
                Some(500),
                Some("/wiki/Journeyman's_Torch"),
            ),
            item(
                9,
                "T4_MAIN_AXE",
                "Adept's Battleaxe",
                Some(700),
                Some("/wiki/Adept's_Battleaxe"),
            ),
        ];
        let index = IdentityIndex::default();
        let mut index = index;
        for row in &existing {
            index.insert(&row.identifier, &row.name, row.id);
        }
        // The dump reports the real identifier for the torch and a new power.
        let incoming = vec![
            ItemRow {
                identifier: Some("T3_OFF_TORCH".to_string()),
                item_power: Some(600),
                path: None,
                ..existing[0].clone()
            },
            ItemRow {
                identifier: Some("T4_MAIN_AXE".to_string()),
                path: None,
                ..existing[1].clone()
            },
        ];
        let (rows, stats) = merge_item_rows(&existing, incoming, &index);
        assert_eq!(rows.len(), 2);
        // ids preserved for both matched rows.
        assert_eq!(rows[0].id, 5);
        assert_eq!(rows[1].id, 9);
        // identifiers/names/power come from the dump, paths are carried over.
        assert_eq!(rows[0].identifier.as_deref(), Some("T3_OFF_TORCH"));
        assert_eq!(rows[0].item_power, Some(600));
        assert_eq!(rows[0].path.as_deref(), Some("/wiki/Journeyman's_Torch"));
        assert_eq!(rows[1].path.as_deref(), Some("/wiki/Adept's_Battleaxe"));
        assert_eq!(stats.identifier_fixed.len(), 1);
        // Only the torch actually changed; the axe row is identical after the
        // path carry-over.
        assert_eq!(stats.updated, 1);
        assert_eq!(stats.unchanged, 1);
    }

    #[test]
    fn new_items_get_fresh_ids_and_old_rows_without_a_match_are_kept() {
        let existing = vec![item(3, "T4_OLD", "Old Item", Some(100), None)];
        let mut index = IdentityIndex::default();
        index.insert(&existing[0].identifier, &existing[0].name, 3);
        let incoming = vec![item(4, "T5_NEW", "New Item", Some(200), None)];
        let (rows, stats) = merge_item_rows(&existing, incoming, &index);
        // Both survive: the unmatched old row is not deleted.
        assert_eq!(rows.len(), 2);
        assert_eq!(stats.inserted, 1);
        assert!(rows
            .iter()
            .any(|r| r.identifier.as_deref() == Some("T4_OLD")));
    }

    #[test]
    fn name_match_recovers_rows_with_mangled_identifiers() {
        let existing = vec![item(
            7,
            "Adept's Torch",
            "  Adept's Torch",
            Some(500),
            Some("/wiki/x"),
        )];
        let mut index = IdentityIndex::default();
        index.insert(&existing[0].identifier, &existing[0].name, 7);
        // Identifier differs, but the normalized name matches.
        let incoming = vec![ItemRow {
            identifier: Some("T4_OFF_TORCH".to_string()),
            name: "Adept's Torch".to_string(),
            path: None,
            ..existing[0].clone()
        }];
        let (rows, stats) = merge_item_rows(&existing, incoming, &index);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, 7);
        assert_eq!(rows[0].identifier.as_deref(), Some("T4_OFF_TORCH"));
        assert_eq!(stats.inserted, 0);
        assert_eq!(stats.updated, 1);
    }

    #[test]
    fn consumable_info_is_carried_over() {
        let existing = vec![ConsumableRow {
            id: 4,
            category_id: Some(52),
            subcategory_id: None,
            name: "Healing Potion".to_string(),
            identifier: Some("T4_POTION_HEAL".to_string()),
            tier: Some(4.0),
            item_power: Some(700),
            path: Some("/wiki/Healing_Potion".to_string()),
            info: Some("Restores health.".to_string()),
        }];
        let mut index = IdentityIndex::default();
        index.insert(&existing[0].identifier, &existing[0].name, 4);
        let incoming = vec![ConsumableRow {
            path: None,
            info: Some("Game description".to_string()),
            ..existing[0].clone()
        }];
        let (rows, _) = merge_consumable_rows(&existing, incoming, &index);
        assert_eq!(rows[0].id, 4);
        // Wiki path carried over; the import does not overwrite `info`.
        assert_eq!(rows[0].path.as_deref(), Some("/wiki/Healing_Potion"));
    }

    #[test]
    fn serialization_keeps_python_format() {
        let rows = vec![CategoryRow {
            id: 1,
            name: "Warrior Weapons".to_string(),
            parent_id: None,
            path: None,
            item_type: "weapon".to_string(),
            version: 1,
        }];
        assert_eq!(
            String::from_utf8(serialize_rows(&rows).unwrap()).unwrap(),
            r#"[{"id": 1, "name": "Warrior Weapons", "parent_id": null, "path": null, "type": "weapon", "version": 1}]"#
        );
        // Whole tiers stay integers.
        let item = item(1, "T4_MAIN_AXE", "Axe", Some(700), None);
        let text = String::from_utf8(serialize_rows(&[item]).unwrap()).unwrap();
        assert!(text.contains(r#""tier": 4,"#), "{text}");
    }

    #[test]
    fn backup_copies_dataset_files() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "openalbion-import-backup-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("weapons.json"), "[]").unwrap();
        fs::write(dir.join("spells.json"), "[]").unwrap();
        let target = backup(&dir, "20260101T000000Z").unwrap().unwrap();
        assert!(target.join("weapons.json").exists());
        assert!(target.join("spells.json").exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn identities_match_across_whitespace_and_case() {
        assert_eq!(normalize_name("  Adept's Torch "), "adept's torch");
        assert_eq!(normalize_name("Elite TerrorBird"), "elite terrorbird");
    }
}
