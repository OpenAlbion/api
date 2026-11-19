//! Serde structs mirroring the exported JSON tables in `data/`.
//!
//! Field order does not matter here (input only) — output ordering lives in
//! `dto.rs`.

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    #[allow(dead_code)] // part of the dataset, never exposed by the API
    pub path: Option<String>,
    #[serde(rename = "type")]
    pub item_type: String,
    pub version: i64,
}

/// Shared shape of `weapons`, `armors` and `accessories` tables.
#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    pub id: i64,
    pub category_id: Option<i64>,
    pub subcategory_id: Option<i64>,
    pub name: String,
    pub identifier: String,
    pub tier: Option<f64>,
    pub item_power: Option<i64>,
    #[allow(dead_code)]
    pub path: Option<String>,
}

/// `consumables` table (adds `info`).
#[derive(Debug, Clone, Deserialize)]
pub struct Consumable {
    pub id: i64,
    pub category_id: Option<i64>,
    pub subcategory_id: Option<i64>,
    pub name: String,
    pub identifier: String,
    pub tier: Option<f64>,
    pub item_power: Option<i64>,
    #[allow(dead_code)]
    pub path: Option<String>,
    pub info: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Spell {
    pub id: i64,
    pub name: String,
    pub identifier: String,
    pub slot: String,
    pub attributes: Value,
    pub description: Option<String>,
    pub preview: Option<String>,
    #[serde(rename = "ref")]
    #[allow(dead_code)]
    pub ref_: String,
}

/// Rows of `weapon_spells` / `armor_spells` / `accessory_spells`. The owning
/// item id column is normalized to `item_id` at load time.
#[derive(Debug, Clone, Deserialize)]
pub struct SpellPivot {
    #[allow(dead_code)]
    pub id: i64,
    #[serde(rename = "weapon_id", alias = "armor_id", alias = "accessory_id")]
    pub item_id: i64,
    pub spell_id: i64,
}

/// Rows of `weapon_stats` / `armor_stats` / `accessory_stats` /
/// `consumable_stats`. The owning item id column is normalized to `item_id`
/// at load time.
#[derive(Debug, Clone, Deserialize)]
pub struct StatRow {
    pub id: i64,
    #[serde(rename = "weapon_id", alias = "armor_id", alias = "accessory_id", alias = "consumable_id")]
    pub item_id: i64,
    pub quality: String,
    pub enchantment: i64,
    pub stats: Value,
}

/// Rows of `consumable_craftings`.
#[derive(Debug, Clone, Deserialize)]
pub struct CraftingRow {
    pub id: i64,
    #[serde(rename = "consumable_id")]
    pub item_id: i64,
    pub per_craft: Option<i64>,
    pub enchantment: i64,
    pub requirements: Value,
}
