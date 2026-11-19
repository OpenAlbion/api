//! The file-backed dataset store.
//!
//! Mirrors what the Laravel commands did through Eloquent `updateOrCreate`:
//! rows are **merged** into the dataset (matched by the same keys the actions
//! used), rows that never show up again are kept, and new rows get
//! `id = max + 1`. Files are written in the exact format `export_sqlite.py`
//! produced (same separators, same key order, whole-number tiers as ints) so
//! that untouched tables stay byte-identical.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::fetch::text::decode_path;

// ---------------------------------------------------------------------------
// Tables and rows
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TableId {
    Categories,
    Weapons,
    Armors,
    Accessories,
    Consumables,
    Spells,
    WeaponSpells,
    ArmorSpells,
    AccessorySpells,
    WeaponStats,
    ArmorStats,
    AccessoryStats,
    ConsumableStats,
    ConsumableCraftings,
}

impl TableId {
    pub const ALL: [TableId; 14] = [
        TableId::Categories,
        TableId::Weapons,
        TableId::Armors,
        TableId::Accessories,
        TableId::Consumables,
        TableId::Spells,
        TableId::WeaponSpells,
        TableId::ArmorSpells,
        TableId::AccessorySpells,
        TableId::WeaponStats,
        TableId::ArmorStats,
        TableId::AccessoryStats,
        TableId::ConsumableStats,
        TableId::ConsumableCraftings,
    ];

    pub fn file(self) -> &'static str {
        match self {
            TableId::Categories => "categories.json",
            TableId::Weapons => "weapons.json",
            TableId::Armors => "armors.json",
            TableId::Accessories => "accessories.json",
            TableId::Consumables => "consumables.json",
            TableId::Spells => "spells.json",
            TableId::WeaponSpells => "weapon_spells.json",
            TableId::ArmorSpells => "armor_spells.json",
            TableId::AccessorySpells => "accessory_spells.json",
            TableId::WeaponStats => "weapon_stats.json",
            TableId::ArmorStats => "armor_stats.json",
            TableId::AccessoryStats => "accessory_stats.json",
            TableId::ConsumableStats => "consumable_stats.json",
            TableId::ConsumableCraftings => "consumable_craftings.json",
        }
    }
}

/// Writes whole-number tiers as integers (`4` not `4.0`) like the SQLite
/// export did, and anything fractional as a plain float.
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellRow {
    pub id: i64,
    pub name: String,
    pub identifier: String,
    pub slot: String,
    pub attributes: Value,
    pub description: Option<String>,
    pub preview: Option<String>,
    #[serde(rename = "ref")]
    pub reference: String,
}

macro_rules! pivot_row {
    ($name:ident, $column:literal) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $name {
            pub id: i64,
            #[serde(rename = $column)]
            pub item_id: i64,
            pub spell_id: i64,
        }
    };
}

pivot_row!(WeaponSpellRow, "weapon_id");
pivot_row!(ArmorSpellRow, "armor_id");
pivot_row!(AccessorySpellRow, "accessory_id");

macro_rules! stat_row {
    ($name:ident, $column:literal) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $name {
            pub id: i64,
            #[serde(rename = $column)]
            pub item_id: i64,
            pub quality: String,
            pub enchantment: i64,
            pub stats: Value,
        }
    };
}

stat_row!(WeaponStatRow, "weapon_id");
stat_row!(ArmorStatRow, "armor_id");
stat_row!(AccessoryStatRow, "accessory_id");
stat_row!(ConsumableStatRow, "consumable_id");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingRow {
    pub id: i64,
    pub consumable_id: i64,
    pub per_craft: Option<i64>,
    pub enchantment: i64,
    pub requirements: Value,
}

#[derive(Debug, Clone, Default)]
pub struct Tables {
    pub categories: Vec<CategoryRow>,
    pub weapons: Vec<ItemRow>,
    pub armors: Vec<ItemRow>,
    pub accessories: Vec<ItemRow>,
    pub consumables: Vec<ConsumableRow>,
    pub spells: Vec<SpellRow>,
    pub weapon_spells: Vec<WeaponSpellRow>,
    pub armor_spells: Vec<ArmorSpellRow>,
    pub accessory_spells: Vec<AccessorySpellRow>,
    pub weapon_stats: Vec<WeaponStatRow>,
    pub armor_stats: Vec<ArmorStatRow>,
    pub accessory_stats: Vec<AccessoryStatRow>,
    pub consumable_stats: Vec<ConsumableStatRow>,
    pub consumable_craftings: Vec<CraftingRow>,
}

impl Tables {
    /// Loads every table from `dir`; missing files are treated as empty
    /// tables (first bootstrap).
    pub fn load(dir: &Path) -> io::Result<Self> {
        Ok(Self {
            categories: read_table(dir, TableId::Categories)?,
            weapons: read_table(dir, TableId::Weapons)?,
            armors: read_table(dir, TableId::Armors)?,
            accessories: read_table(dir, TableId::Accessories)?,
            consumables: read_table(dir, TableId::Consumables)?,
            spells: read_table(dir, TableId::Spells)?,
            weapon_spells: read_table(dir, TableId::WeaponSpells)?,
            armor_spells: read_table(dir, TableId::ArmorSpells)?,
            accessory_spells: read_table(dir, TableId::AccessorySpells)?,
            weapon_stats: read_table(dir, TableId::WeaponStats)?,
            armor_stats: read_table(dir, TableId::ArmorStats)?,
            accessory_stats: read_table(dir, TableId::AccessoryStats)?,
            consumable_stats: read_table(dir, TableId::ConsumableStats)?,
            consumable_craftings: read_table(dir, TableId::ConsumableCraftings)?,
        })
    }

    /// Human key set of one table, used by the diff report.
    pub fn keys(&self, table: TableId) -> BTreeSet<String> {
        match table {
            TableId::Categories => self.categories.iter().map(|r| r.name.clone()).collect(),
            TableId::Weapons => self.weapons.iter().map(|r| r.name.clone()).collect(),
            TableId::Armors => self.armors.iter().map(|r| r.name.clone()).collect(),
            TableId::Accessories => self.accessories.iter().map(|r| r.name.clone()).collect(),
            TableId::Consumables => self.consumables.iter().map(|r| r.name.clone()).collect(),
            TableId::Spells => self.spells.iter().map(|r| r.name.clone()).collect(),
            TableId::WeaponSpells => self
                .weapon_spells
                .iter()
                .map(|r| {
                    pivot_key(
                        self.item_name(TableId::Weapons, r.item_id),
                        self.spell_name(r.spell_id),
                    )
                })
                .collect(),
            TableId::ArmorSpells => self
                .armor_spells
                .iter()
                .map(|r| {
                    pivot_key(
                        self.item_name(TableId::Armors, r.item_id),
                        self.spell_name(r.spell_id),
                    )
                })
                .collect(),
            TableId::AccessorySpells => self
                .accessory_spells
                .iter()
                .map(|r| {
                    pivot_key(
                        self.item_name(TableId::Accessories, r.item_id),
                        self.spell_name(r.spell_id),
                    )
                })
                .collect(),
            TableId::WeaponStats => self
                .weapon_stats
                .iter()
                .map(|r| {
                    stat_key(
                        self.item_name(TableId::Weapons, r.item_id),
                        &r.quality,
                        r.enchantment,
                    )
                })
                .collect(),
            TableId::ArmorStats => self
                .armor_stats
                .iter()
                .map(|r| {
                    stat_key(
                        self.item_name(TableId::Armors, r.item_id),
                        &r.quality,
                        r.enchantment,
                    )
                })
                .collect(),
            TableId::AccessoryStats => self
                .accessory_stats
                .iter()
                .map(|r| {
                    stat_key(
                        self.item_name(TableId::Accessories, r.item_id),
                        &r.quality,
                        r.enchantment,
                    )
                })
                .collect(),
            TableId::ConsumableStats => self
                .consumable_stats
                .iter()
                .map(|r| {
                    stat_key(
                        self.item_name(TableId::Consumables, r.item_id),
                        &r.quality,
                        r.enchantment,
                    )
                })
                .collect(),
            TableId::ConsumableCraftings => self
                .consumable_craftings
                .iter()
                .map(|r| {
                    crafting_key(
                        self.item_name(TableId::Consumables, r.consumable_id),
                        r.per_craft,
                        r.enchantment,
                    )
                })
                .collect(),
        }
    }

    pub fn len(&self, table: TableId) -> usize {
        match table {
            TableId::Categories => self.categories.len(),
            TableId::Weapons => self.weapons.len(),
            TableId::Armors => self.armors.len(),
            TableId::Accessories => self.accessories.len(),
            TableId::Consumables => self.consumables.len(),
            TableId::Spells => self.spells.len(),
            TableId::WeaponSpells => self.weapon_spells.len(),
            TableId::ArmorSpells => self.armor_spells.len(),
            TableId::AccessorySpells => self.accessory_spells.len(),
            TableId::WeaponStats => self.weapon_stats.len(),
            TableId::ArmorStats => self.armor_stats.len(),
            TableId::AccessoryStats => self.accessory_stats.len(),
            TableId::ConsumableStats => self.consumable_stats.len(),
            TableId::ConsumableCraftings => self.consumable_craftings.len(),
        }
    }

    pub fn is_empty_table(&self, table: TableId) -> bool {
        self.len(table) == 0
    }

    fn item_name(&self, table: TableId, id: i64) -> String {
        match table {
            TableId::Weapons => self
                .weapons
                .iter()
                .find(|r| r.id == id)
                .map(|r| r.name.clone()),
            TableId::Armors => self
                .armors
                .iter()
                .find(|r| r.id == id)
                .map(|r| r.name.clone()),
            TableId::Accessories => self
                .accessories
                .iter()
                .find(|r| r.id == id)
                .map(|r| r.name.clone()),
            TableId::Consumables => self
                .consumables
                .iter()
                .find(|r| r.id == id)
                .map(|r| r.name.clone()),
            _ => None,
        }
        .unwrap_or_else(|| format!("#{id}"))
    }

    fn spell_name(&self, id: i64) -> String {
        self.spells
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| format!("#{id}"))
    }
}

pub fn pivot_key(item: String, spell: String) -> String {
    format!("{item} -> {spell}")
}

pub fn stat_key(item: String, quality: &str, enchantment: i64) -> String {
    format!("{item} | {quality} | e{enchantment}")
}

pub fn crafting_key(item: String, per_craft: Option<i64>, enchantment: i64) -> String {
    format!(
        "{item} | per_craft {} | e{enchantment}",
        per_craft.unwrap_or(0)
    )
}

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

/// `serde_json` formatter matching Python's `json.dumps` defaults
/// (`separators=(", ", ": ")`, `ensure_ascii=False`), which is how
/// `export_sqlite.py` wrote the files.
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

fn serialize_rows<T: Serialize>(rows: &[T]) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut buffer, PythonFormatter);
    rows.serialize(&mut serializer).map_err(io::Error::other)?;
    Ok(buffer)
}

fn read_table<T: for<'de> Deserialize<'de>>(dir: &Path, table: TableId) -> io::Result<Vec<T>> {
    let path = dir.join(table.file());
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path)
        .map_err(|e| io::Error::new(e.kind(), format!("cannot read {}: {e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid JSON in {}: {e}", path.display()),
        )
    })
}

fn write_table<T: Serialize>(dir: &Path, table: TableId, rows: &[T]) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let bytes = serialize_rows(rows)?;
    let path = dir.join(table.file());
    let tmp = dir.join(format!(".{}.tmp", table.file()));
    fs::write(&tmp, &bytes)?;
    fs::rename(&tmp, &path)
}

// ---------------------------------------------------------------------------
// The store
// ---------------------------------------------------------------------------

/// Holds the working dataset plus what the run has seen so far.
pub struct Store {
    pub dir: PathBuf,
    /// Snapshot of the dataset as it was before this run, for the diff.
    pub old: Tables,
    pub tables: Tables,
    /// Keys produced by this run, per table (drives added/missing reporting).
    fetched: HashMap<TableId, BTreeSet<String>>,
    /// Tables whose rows changed and therefore need rewriting.
    touched: HashSet<TableId>,
    /// Tables this command only covers a subset of (a cape crawl does not
    /// see every accessory), so "not returned by this fetch" must not be
    /// reported as a missing row.
    partial: HashSet<TableId>,
}

impl Store {
    pub fn open(dir: &Path) -> io::Result<Self> {
        let tables = Tables::load(dir)?;
        Ok(Self {
            dir: dir.to_path_buf(),
            old: tables.clone(),
            tables,
            fetched: HashMap::new(),
            touched: HashSet::new(),
            partial: HashSet::new(),
        })
    }

    /// Records a row key as "seen by this fetch" and marks the table dirty.
    fn mark(&mut self, table: TableId, key: String) {
        self.fetched.entry(table).or_default().insert(key);
        self.touched.insert(table);
    }

    pub fn fetched_keys(&self) -> &HashMap<TableId, BTreeSet<String>> {
        &self.fetched
    }

    pub fn touched_tables(&self) -> Vec<TableId> {
        let mut tables: Vec<TableId> = self.touched.iter().copied().collect();
        tables.sort();
        tables
    }

    /// Marks a table dirty without recording fetched keys (fixup commands).
    pub fn touch(&mut self, table: TableId) {
        self.touched.insert(table);
    }

    /// Records that a fetch ran for a table without producing any rows.
    /// Used by the degenerate-fetch guard (and its tests) so a broken parser
    /// can never shrink a table.
    pub fn note_fetch_attempt(&mut self, table: TableId) {
        self.fetched.entry(table).or_default();
        self.touched.insert(table);
    }

    /// Marks a table as only partially covered by this command (e.g. the cape
    /// crawl sees the cape rows of the accessory table, not all of them), so
    /// the report does not flag the untouched rows as "missing".
    pub fn mark_partial(&mut self, table: TableId) {
        self.partial.insert(table);
    }

    pub fn partial_tables(&self) -> &HashSet<TableId> {
        &self.partial
    }

    /// Undoes a [`Self::touch`] (used by the degenerate-fetch guard).
    pub fn unmark_table(&mut self, table: TableId) {
        self.touched.remove(&table);
    }

    pub fn fetched_count(&self, table: TableId) -> usize {
        self.fetched.get(&table).map(BTreeSet::len).unwrap_or(0)
    }

    pub fn fetched_any(&self, table: TableId) -> bool {
        self.fetched.contains_key(&table)
    }

    pub fn has_changes(&self) -> bool {
        !self.touched.is_empty()
    }

    /// Writes all touched tables (no-op for untouched ones).
    pub fn save(&self) -> io::Result<()> {
        let dir = &self.dir;
        for table in self.touched_tables() {
            match table {
                TableId::Categories => write_table(dir, table, &self.tables.categories)?,
                TableId::Weapons => write_table(dir, table, &self.tables.weapons)?,
                TableId::Armors => write_table(dir, table, &self.tables.armors)?,
                TableId::Accessories => write_table(dir, table, &self.tables.accessories)?,
                TableId::Consumables => write_table(dir, table, &self.tables.consumables)?,
                TableId::Spells => write_table(dir, table, &self.tables.spells)?,
                TableId::WeaponSpells => write_table(dir, table, &self.tables.weapon_spells)?,
                TableId::ArmorSpells => write_table(dir, table, &self.tables.armor_spells)?,
                TableId::AccessorySpells => write_table(dir, table, &self.tables.accessory_spells)?,
                TableId::WeaponStats => write_table(dir, table, &self.tables.weapon_stats)?,
                TableId::ArmorStats => write_table(dir, table, &self.tables.armor_stats)?,
                TableId::AccessoryStats => write_table(dir, table, &self.tables.accessory_stats)?,
                TableId::ConsumableStats => write_table(dir, table, &self.tables.consumable_stats)?,
                TableId::ConsumableCraftings => {
                    write_table(dir, table, &self.tables.consumable_craftings)?
                }
            }
        }
        Ok(())
    }

    // -- lookups ---------------------------------------------------------

    /// Subcategories of a category type that have a wiki path.
    pub fn subcategories_with_path(&self, item_type: &str) -> Vec<(i64, i64, String)> {
        self.tables
            .categories
            .iter()
            .filter(|c| c.item_type == item_type)
            .filter(|c| c.path.is_some())
            .filter_map(|c| {
                let parent = c.parent_id?;
                Some((c.id, parent, c.path.clone()?))
            })
            .collect()
    }

    /// Items of one kind that have a wiki path, as `(id, path)`.
    pub fn items_with_path(&self, table: TableId) -> Vec<(i64, String)> {
        let rows: Vec<(i64, Option<&String>)> = match table {
            TableId::Weapons => self
                .tables
                .weapons
                .iter()
                .map(|r| (r.id, r.path.as_ref()))
                .collect(),
            TableId::Armors => self
                .tables
                .armors
                .iter()
                .map(|r| (r.id, r.path.as_ref()))
                .collect(),
            TableId::Accessories => self
                .tables
                .accessories
                .iter()
                .map(|r| (r.id, r.path.as_ref()))
                .collect(),
            TableId::Consumables => self
                .tables
                .consumables
                .iter()
                .map(|r| (r.id, r.path.as_ref()))
                .collect(),
            other => unreachable!("items_with_path called for {other:?}"),
        };
        rows.into_iter()
            .filter_map(|(id, path)| Some((id, path?.clone())))
            .collect()
    }

    pub fn category_by_name(&self, name: &str) -> Option<&CategoryRow> {
        self.tables.categories.iter().find(|r| r.name == name)
    }

    pub fn category_by_id(&self, id: i64) -> Option<&CategoryRow> {
        self.tables.categories.iter().find(|r| r.id == id)
    }

    pub fn category_children(&self, parent_id: i64) -> Vec<&CategoryRow> {
        self.tables
            .categories
            .iter()
            .filter(|r| r.parent_id == Some(parent_id))
            .collect()
    }

    pub fn weapons_with_path(&self) -> Vec<&ItemRow> {
        self.tables
            .weapons
            .iter()
            .filter(|r| r.path.is_some())
            .collect()
    }

    pub fn armors_with_path(&self) -> Vec<&ItemRow> {
        self.tables
            .armors
            .iter()
            .filter(|r| r.path.is_some())
            .collect()
    }

    pub fn accessories_with_path(&self) -> Vec<&ItemRow> {
        self.tables
            .accessories
            .iter()
            .filter(|r| r.path.is_some())
            .collect()
    }

    pub fn consumables_with_path(&self) -> Vec<&ConsumableRow> {
        self.tables
            .consumables
            .iter()
            .filter(|r| r.path.is_some())
            .collect()
    }

    // -- upserts ---------------------------------------------------------

    /// `UpdateCategory`: match by name; create the row when missing.
    pub fn upsert_category(
        &mut self,
        name: &str,
        parent_id: Option<i64>,
        path: Option<&str>,
        item_type: &str,
        version: i64,
    ) -> i64 {
        let decoded_path = path.and_then(decode_path);
        let id = if let Some(row) = self.tables.categories.iter_mut().find(|r| r.name == name) {
            row.parent_id = parent_id;
            if decoded_path.is_some() {
                row.path = decoded_path;
            }
            row.item_type = item_type.to_string();
            row.version = version;
            row.id
        } else {
            let id = next_id(&self.tables.categories, |r| r.id);
            self.tables.categories.push(CategoryRow {
                id,
                name: name.to_string(),
                parent_id,
                path: decoded_path,
                item_type: item_type.to_string(),
                version,
            });
            id
        };
        self.mark(TableId::Categories, name.to_string());
        id
    }

    /// `UpdateWeapon` / `UpdateArmor` / `UpdateAccessory`: match by name,
    /// merge fields (null-safe: a `None` never overwrites a value).
    #[allow(clippy::too_many_arguments)]
    pub fn upsert_item(
        &mut self,
        table: TableId,
        name: &str,
        category_id: Option<i64>,
        subcategory_id: Option<i64>,
        identifier: Option<&str>,
        path: Option<&str>,
        tier: Option<f64>,
        item_power: Option<i64>,
    ) -> i64 {
        let decoded_path = path.and_then(decode_path);
        let rows = match table {
            TableId::Weapons => &mut self.tables.weapons,
            TableId::Armors => &mut self.tables.armors,
            TableId::Accessories => &mut self.tables.accessories,
            other => unreachable!("upsert_item called for {other:?}"),
        };
        let id = if let Some(row) = rows.iter_mut().find(|r| r.name == name) {
            row.category_id = category_id.or(row.category_id);
            row.subcategory_id = subcategory_id.or(row.subcategory_id);
            row.identifier = identifier
                .map(str::to_string)
                .or_else(|| row.identifier.clone());
            if decoded_path.is_some() {
                row.path = decoded_path;
            }
            row.tier = tier.or(row.tier);
            row.item_power = item_power.or(row.item_power);
            row.id
        } else {
            let id = next_id(rows, |r| r.id);
            rows.push(ItemRow {
                id,
                category_id,
                subcategory_id,
                name: name.to_string(),
                identifier: identifier.map(str::to_string),
                tier,
                item_power,
                path: decoded_path,
            });
            id
        };
        self.mark(table, name.to_string());
        id
    }

    /// `UpdateConsumable`: same as [`Self::upsert_item`] plus `info`.
    #[allow(clippy::too_many_arguments)]
    pub fn upsert_consumable(
        &mut self,
        name: &str,
        category_id: Option<i64>,
        subcategory_id: Option<i64>,
        identifier: Option<&str>,
        path: Option<&str>,
        tier: Option<f64>,
        item_power: Option<i64>,
    ) -> i64 {
        let decoded_path = path.and_then(decode_path);
        let rows = &mut self.tables.consumables;
        let id = if let Some(row) = rows.iter_mut().find(|r| r.name == name) {
            row.category_id = category_id.or(row.category_id);
            row.subcategory_id = subcategory_id.or(row.subcategory_id);
            row.identifier = identifier
                .map(str::to_string)
                .or_else(|| row.identifier.clone());
            if decoded_path.is_some() {
                row.path = decoded_path;
            }
            row.tier = tier.or(row.tier);
            row.item_power = item_power.or(row.item_power);
            row.id
        } else {
            let id = next_id(rows, |r| r.id);
            rows.push(ConsumableRow {
                id,
                category_id,
                subcategory_id,
                name: name.to_string(),
                identifier: identifier.map(str::to_string),
                tier,
                item_power,
                path: decoded_path,
                info: None,
            });
            id
        };
        self.mark(TableId::Consumables, name.to_string());
        id
    }

    /// `UpdateSpell`: match by name, replace the parsed fields.
    #[allow(clippy::too_many_arguments)]
    pub fn upsert_spell(
        &mut self,
        name: &str,
        identifier: Option<&str>,
        slot: &str,
        attributes: Value,
        description: Option<&str>,
        preview: Option<&str>,
        reference: &str,
    ) -> i64 {
        let rows = &mut self.tables.spells;
        let id = if let Some(row) = rows.iter_mut().find(|r| r.name == name) {
            row.identifier = identifier
                .map(str::to_string)
                .unwrap_or_else(|| row.identifier.clone());
            row.slot = slot.to_string();
            row.attributes = attributes;
            row.description = description.map(str::to_string);
            row.preview = preview.map(str::to_string);
            row.reference = reference.to_string();
            row.id
        } else {
            let id = next_id(rows, |r| r.id);
            rows.push(SpellRow {
                id,
                name: name.to_string(),
                identifier: identifier.unwrap_or_default().to_string(),
                slot: slot.to_string(),
                attributes,
                description: description.map(str::to_string),
                preview: preview.map(str::to_string),
                reference: reference.to_string(),
            });
            id
        };
        self.mark(TableId::Spells, name.to_string());
        id
    }

    /// Links an item to a spell (idempotent, like the pivot `updateOrCreate`).
    pub fn link_spell(&mut self, table: TableId, item_id: i64, spell_id: i64) {
        let (item_key, spell_key) = (
            self.item_key_name(table, item_id),
            self.spell_name(spell_id),
        );
        match table {
            TableId::WeaponSpells => {
                if !self
                    .tables
                    .weapon_spells
                    .iter()
                    .any(|r| r.item_id == item_id && r.spell_id == spell_id)
                {
                    let id = next_id(&self.tables.weapon_spells, |r| r.id);
                    self.tables.weapon_spells.push(WeaponSpellRow {
                        id,
                        item_id,
                        spell_id,
                    });
                }
            }
            TableId::ArmorSpells => {
                if !self
                    .tables
                    .armor_spells
                    .iter()
                    .any(|r| r.item_id == item_id && r.spell_id == spell_id)
                {
                    let id = next_id(&self.tables.armor_spells, |r| r.id);
                    self.tables.armor_spells.push(ArmorSpellRow {
                        id,
                        item_id,
                        spell_id,
                    });
                }
            }
            TableId::AccessorySpells => {
                if !self
                    .tables
                    .accessory_spells
                    .iter()
                    .any(|r| r.item_id == item_id && r.spell_id == spell_id)
                {
                    let id = next_id(&self.tables.accessory_spells, |r| r.id);
                    self.tables.accessory_spells.push(AccessorySpellRow {
                        id,
                        item_id,
                        spell_id,
                    });
                }
            }
            other => unreachable!("link_spell called for {other:?}"),
        }
        self.mark(table, pivot_key(item_key, spell_key));
    }

    /// `UpdateWeaponStat` & friends: match by (item, quality, enchantment).
    pub fn upsert_stat(
        &mut self,
        table: TableId,
        item_id: i64,
        quality: &str,
        enchantment: i64,
        stats: Value,
    ) {
        let item_key = self.item_key_name(table, item_id);
        match table {
            TableId::WeaponStats => {
                if let Some(row) = self.tables.weapon_stats.iter_mut().find(|r| {
                    r.item_id == item_id && r.quality == quality && r.enchantment == enchantment
                }) {
                    row.stats = stats;
                } else {
                    let id = next_id(&self.tables.weapon_stats, |r| r.id);
                    self.tables.weapon_stats.push(WeaponStatRow {
                        id,
                        item_id,
                        quality: quality.to_string(),
                        enchantment,
                        stats,
                    });
                }
            }
            TableId::ArmorStats => {
                if let Some(row) = self.tables.armor_stats.iter_mut().find(|r| {
                    r.item_id == item_id && r.quality == quality && r.enchantment == enchantment
                }) {
                    row.stats = stats;
                } else {
                    let id = next_id(&self.tables.armor_stats, |r| r.id);
                    self.tables.armor_stats.push(ArmorStatRow {
                        id,
                        item_id,
                        quality: quality.to_string(),
                        enchantment,
                        stats,
                    });
                }
            }
            TableId::AccessoryStats => {
                if let Some(row) = self.tables.accessory_stats.iter_mut().find(|r| {
                    r.item_id == item_id && r.quality == quality && r.enchantment == enchantment
                }) {
                    row.stats = stats;
                } else {
                    let id = next_id(&self.tables.accessory_stats, |r| r.id);
                    self.tables.accessory_stats.push(AccessoryStatRow {
                        id,
                        item_id,
                        quality: quality.to_string(),
                        enchantment,
                        stats,
                    });
                }
            }
            TableId::ConsumableStats => {
                if let Some(row) = self.tables.consumable_stats.iter_mut().find(|r| {
                    r.item_id == item_id && r.quality == quality && r.enchantment == enchantment
                }) {
                    row.stats = stats;
                } else {
                    let id = next_id(&self.tables.consumable_stats, |r| r.id);
                    self.tables.consumable_stats.push(ConsumableStatRow {
                        id,
                        item_id,
                        quality: quality.to_string(),
                        enchantment,
                        stats,
                    });
                }
            }
            other => unreachable!("upsert_stat called for {other:?}"),
        }
        self.mark(table, stat_key(item_key, quality, enchantment));
    }

    /// `UpdateConsumableCrafting`: match by (consumable, per_craft, enchantment).
    pub fn upsert_crafting(
        &mut self,
        consumable_id: i64,
        per_craft: Option<i64>,
        enchantment: i64,
        requirements: Value,
    ) {
        let key = crafting_key(
            self.item_key_name(TableId::Consumables, consumable_id),
            per_craft,
            enchantment,
        );
        let rows = &mut self.tables.consumable_craftings;
        if let Some(row) = rows.iter_mut().find(|r| {
            r.consumable_id == consumable_id
                && r.per_craft == per_craft
                && r.enchantment == enchantment
        }) {
            row.requirements = requirements;
        } else {
            let id = next_id(rows, |r| r.id);
            rows.push(CraftingRow {
                id,
                consumable_id,
                per_craft,
                enchantment,
                requirements,
            });
        }
        self.mark(TableId::ConsumableCraftings, key);
    }

    /// Sets `info` on a consumable (the `GetConsumableInfo` command).
    pub fn set_consumable_info(&mut self, id: i64, info: &str) {
        if let Some(row) = self.tables.consumables.iter_mut().find(|r| r.id == id) {
            row.info = Some(info.to_string());
        }
        self.touched.insert(TableId::Consumables);
    }

    fn item_key_name(&self, table: TableId, item_id: i64) -> String {
        let name = match table {
            TableId::WeaponSpells | TableId::WeaponStats => self
                .tables
                .weapons
                .iter()
                .find(|r| r.id == item_id)
                .map(|r| r.name.clone()),
            TableId::ArmorSpells | TableId::ArmorStats => self
                .tables
                .armors
                .iter()
                .find(|r| r.id == item_id)
                .map(|r| r.name.clone()),
            TableId::AccessorySpells | TableId::AccessoryStats => self
                .tables
                .accessories
                .iter()
                .find(|r| r.id == item_id)
                .map(|r| r.name.clone()),
            TableId::ConsumableStats | TableId::ConsumableCraftings => self
                .tables
                .consumables
                .iter()
                .find(|r| r.id == item_id)
                .map(|r| r.name.clone()),
            TableId::Weapons => self
                .tables
                .weapons
                .iter()
                .find(|r| r.id == item_id)
                .map(|r| r.name.clone()),
            TableId::Armors => self
                .tables
                .armors
                .iter()
                .find(|r| r.id == item_id)
                .map(|r| r.name.clone()),
            TableId::Accessories => self
                .tables
                .accessories
                .iter()
                .find(|r| r.id == item_id)
                .map(|r| r.name.clone()),
            TableId::Consumables => self
                .tables
                .consumables
                .iter()
                .find(|r| r.id == item_id)
                .map(|r| r.name.clone()),
            _ => None,
        };
        name.unwrap_or_else(|| format!("#{item_id}"))
    }

    fn spell_name(&self, id: i64) -> String {
        self.tables
            .spells
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| format!("#{id}"))
    }
}

fn next_id<T>(rows: &[T], id: impl Fn(&T) -> i64) -> i64 {
    rows.iter().map(id).max().unwrap_or(0) + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "openalbion-store-{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn category_upsert_matches_by_name_and_allocates_ids() {
        let dir = temp_dir("cat");
        let mut store = Store::open(&dir).unwrap();
        let a = store.upsert_category("Weapons", None, None, "weapon", 1);
        let b = store.upsert_category("Swords", Some(a), Some("/wiki/Sword"), "weapon", 1);
        assert_eq!((a, b), (1, 2));
        // Same name updates in place.
        let again = store.upsert_category("Swords", Some(a), Some("/wiki/Swords"), "weapon", 1);
        assert_eq!(again, 2);
        assert_eq!(store.tables.categories.len(), 2);
        assert_eq!(
            store.tables.categories[1].path.as_deref(),
            Some("/wiki/Swords")
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn item_upsert_is_null_safe_on_update() {
        let dir = temp_dir("item");
        let mut store = Store::open(&dir).unwrap();
        let id = store.upsert_item(
            TableId::Weapons,
            "Adept's Sword",
            Some(1),
            Some(2),
            Some("T4_SWORD"),
            Some("/wiki/Adept%27s_Sword"),
            Some(4.0),
            Some(700),
        );
        // Re-crawl without an icon/tier must not wipe the stored values.
        let again = store.upsert_item(
            TableId::Weapons,
            "Adept's Sword",
            Some(1),
            Some(2),
            None,
            None,
            None,
            None,
        );
        assert_eq!(id, again);
        let row = &store.tables.weapons[0];
        assert_eq!(row.identifier.as_deref(), Some("T4_SWORD"));
        assert_eq!(row.tier, Some(4.0));
        assert_eq!(row.item_power, Some(700));
        assert_eq!(row.path.as_deref(), Some("/wiki/Adept's_Sword"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn new_rows_append_after_existing_max_id() {
        let dir = temp_dir("ids");
        fs::write(
            dir.join("weapons.json"),
            r#"[{"id": 7, "category_id": 1, "subcategory_id": 2, "name": "Old", "identifier": null, "tier": 4, "item_power": 700, "path": null}]"#,
        )
        .unwrap();
        let mut store = Store::open(&dir).unwrap();
        let id = store.upsert_item(
            TableId::Weapons,
            "New",
            Some(1),
            Some(2),
            None,
            None,
            Some(5.0),
            None,
        );
        assert_eq!(id, 8);
        assert_eq!(store.tables.weapons.len(), 2);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn serialization_matches_python_json_dumps() {
        let rows = vec![
            CategoryRow {
                id: 1,
                name: "Warrior Weapons".to_string(),
                parent_id: None,
                path: None,
                item_type: "weapon".to_string(),
                version: 1,
            },
            CategoryRow {
                id: 2,
                name: "Axe".to_string(),
                parent_id: Some(1),
                path: Some("/wiki/Axe".to_string()),
                item_type: "weapon".to_string(),
                version: 1,
            },
        ];
        let expected = r#"[{"id": 1, "name": "Warrior Weapons", "parent_id": null, "path": null, "type": "weapon", "version": 1}, {"id": 2, "name": "Axe", "parent_id": 1, "path": "/wiki/Axe", "type": "weapon", "version": 1}]"#;
        assert_eq!(
            String::from_utf8(serialize_rows(&rows).unwrap()).unwrap(),
            expected
        );

        // Whole tiers serialise as integers like the SQLite export.
        let item = ItemRow {
            id: 1,
            category_id: Some(1),
            subcategory_id: Some(2),
            name: "Adept's Sword".to_string(),
            identifier: Some("T4_SWORD".to_string()),
            tier: Some(4.0),
            item_power: Some(700),
            path: Some("/wiki/Adept's_Sword".to_string()),
        };
        let text = String::from_utf8(serialize_rows(&[item]).unwrap()).unwrap();
        assert!(text.contains(r#""tier": 4,"#), "{text}");
    }

    #[test]
    fn save_and_reload_roundtrip() {
        let dir = temp_dir("roundtrip");
        let mut store = Store::open(&dir).unwrap();
        store.upsert_item(
            TableId::Weapons,
            "Adept's Sword",
            Some(1),
            Some(2),
            Some("T4_SWORD"),
            Some("/wiki/Adept%27s_Sword"),
            Some(4.0),
            Some(700),
        );
        store.save().unwrap();
        let reloaded = Tables::load(&dir).unwrap();
        assert_eq!(reloaded.weapons.len(), 1);
        assert_eq!(reloaded.weapons[0].identifier.as_deref(), Some("T4_SWORD"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn stat_and_pivot_upserts_are_idempotent() {
        let dir = temp_dir("stats");
        let mut store = Store::open(&dir).unwrap();
        store.upsert_item(
            TableId::Weapons,
            "Sword",
            None,
            None,
            None,
            None,
            None,
            None,
        );
        store.upsert_spell(
            "Slash",
            Some("Slash"),
            "q",
            json!([]),
            Some("desc"),
            None,
            "/wiki/Slash",
        );
        store.upsert_stat(
            TableId::WeaponStats,
            1,
            "Normal",
            0,
            json!([{"name": "Item Power", "value": "700"}]),
        );
        store.upsert_stat(
            TableId::WeaponStats,
            1,
            "Normal",
            0,
            json!([{"name": "Item Power", "value": "700"}]),
        );
        store.link_spell(TableId::WeaponSpells, 1, 1);
        store.link_spell(TableId::WeaponSpells, 1, 1);
        assert_eq!(store.tables.weapon_stats.len(), 1);
        assert_eq!(store.tables.weapon_spells.len(), 1);
        let _ = fs::remove_dir_all(dir);
    }
}
