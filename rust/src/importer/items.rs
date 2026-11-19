//! Turning dump items into dataset rows (the overwrite itself).
//!
//! Field policy (AOD is authoritative, wiki-only fields carried over):
//!
//! | field                     | source                                            |
//! |---------------------------|---------------------------------------------------|
//! | `identifier`              | `@uniquename` (fixes mangled torch/tome ids)      |
//! | `name`                    | `LocalizedNames.EN-US`                            |
//! | `tier`                    | `@tier`                                           |
//! | `item_power`              | `@itempower` / `@dummyitempower`, else inferred   |
//! | `category_id`             | mapped category name                              |
//! | `subcategory_id`          | mapped subcategory (or `null` when unmapped)      |
//! | `path`, consumable `info` | carried over from the existing row (wiki data)    |
//! | `id`                      | preserved for matched rows, `max+1` for new ones  |

use std::collections::HashMap;

use crate::importer::aod::AodDump;
use crate::importer::mapping::{self, Table};
use crate::importer::store::{
    next_id, ConsumableRow, Dataset, IdentityIndex, ItemRow, OverwriteStats,
};

/// An item from the dump that will be written, with its resolved target.
#[derive(Debug, Clone)]
pub struct PlannedItem {
    pub table: Table,
    pub unique_name: String,
    pub name: String,
    pub tier: Option<f64>,
    pub item_power: Option<i64>,
    pub category_id: Option<i64>,
    pub subcategory_id: Option<i64>,
    pub description: Option<String>,
    /// True when the wiki tree has no subcategory for this item.
    pub unmapped_subcategory: bool,
}

/// Items that were deliberately not imported, with the reason.
#[derive(Debug, Clone)]
pub struct SkippedItem {
    pub unique_name: String,
    pub name: String,
    pub reason: &'static str,
}

#[derive(Debug, Default)]
pub struct Plan {
    pub items: Vec<PlannedItem>,
    pub skipped: Vec<SkippedItem>,
    /// Identifiers the dump marks as cosmetic/test — these are also removed
    /// from the dataset if a previous import added them.
    pub excluded_ids: Vec<String>,
    /// Items with no item power anywhere in the dump (new rows only).
    pub missing_item_power: usize,
}

impl Plan {
    pub fn count(&self, table: Table) -> usize {
        self.items.iter().filter(|i| i.table == table).count()
    }
}

/// Most common item power per (table, tier) among existing rows — the dataset's
/// convention (tier 4 weapons are all 700 in the export, while the dump says
/// 600 for T3), used only when the dump has no value.
fn learned_item_power(dataset: &Dataset, table: Table, tier: i64) -> Option<i64> {
    let mut counts: HashMap<i64, usize> = HashMap::new();
    let rows: Vec<&ItemRow> = match table {
        Table::Weapons => dataset.weapons.iter().collect(),
        Table::Armors => dataset.armors.iter().collect(),
        Table::Accessories => dataset.accessories.iter().collect(),
        Table::Consumables => return None,
    };
    for row in rows {
        if row.tier == Some(tier as f64) {
            if let Some(power) = row.item_power {
                *counts.entry(power).or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(power, _)| power)
}

/// Resolves the dump into a list of planned rows.
pub fn plan(dump: &AodDump, dataset: &Dataset, include_cosmetics: bool) -> Plan {
    let index = dataset.identity_index();
    let mut plan = Plan::default();
    let existing_names: std::collections::HashSet<String> = index.existing_names();

    for item in dump.sorted() {
        if item.name.is_empty() {
            plan.skipped.push(SkippedItem {
                unique_name: item.unique_name.clone(),
                name: item.name.clone(),
                reason: "no English name",
            });
            continue;
        }
        let Some(table) = mapping::table_of(item) else {
            // Not a gameplay item (furniture, resources, …) — silently ignored.
            continue;
        };
        if !include_cosmetics && item.is_cosmetic_or_test() {
            plan.excluded_ids.push(item.unique_name.clone());
            plan.skipped.push(SkippedItem {
                unique_name: item.unique_name.clone(),
                name: item.name.clone(),
                reason: "cosmetic/test item",
            });
            continue;
        }
        // Duplicate display names (different unique names) would collide in
        // the dataset, which keys items by name on the wiki side.
        let normalized = crate::importer::store::normalize_name(&item.name);
        if existing_names.contains(&normalized)
            && index.find(&item.unique_name, &item.name).is_none()
        {
            plan.skipped.push(SkippedItem {
                unique_name: item.unique_name.clone(),
                name: item.name.clone(),
                reason: "duplicate display name",
            });
            continue;
        }

        let path = mapping::category_path(item);
        let (category_id, subcategory_id, unmapped) = match &path {
            Some(path) if !path.category.is_empty() => {
                let category_id = dataset.category_id(&path.category);
                let subcategory_id = path
                    .subcategory
                    .as_deref()
                    .and_then(|sub| dataset.subcategory_id(&path.category, sub))
                    .or_else(|| {
                        // Unmapped subcategory: fall back to the same-named
                        // category root when the wiki tree has one.
                        dataset.category_id(&path.category).filter(|_| false)
                    });
                (category_id, subcategory_id, subcategory_id.is_none())
            }
            _ => (None, None, true),
        };

        let tier = item.tier;
        let item_power = item.item_power.or_else(|| {
            let tier = tier?;
            learned_item_power(dataset, table, tier)
        });
        if item_power.is_none() && table != Table::Consumables {
            plan.missing_item_power += 1;
        }

        plan.items.push(PlannedItem {
            table,
            unique_name: item.unique_name.clone(),
            name: item.name.clone(),
            tier: tier.map(|t| t as f64),
            item_power,
            category_id,
            subcategory_id,
            description: item.description.clone(),
            unmapped_subcategory: unmapped,
        });
    }
    plan
}

/// Builds the row structs for one table from the plan.
pub fn rows_for(
    plan: &Plan,
    table: Table,
    dataset: &Dataset,
    index: &IdentityIndex,
) -> Vec<ItemRow> {
    let mut rows = Vec::new();
    let mut next = next_id(dataset.items(table), |r| r.id);
    for item in plan.items.iter().filter(|i| i.table == table) {
        let existing = index.find(&item.unique_name, &item.name);
        let id = existing.unwrap_or_else(|| {
            let id = next;
            next += 1;
            id
        });
        rows.push(ItemRow {
            id,
            category_id: item.category_id,
            subcategory_id: item.subcategory_id,
            name: item.name.clone(),
            identifier: Some(item.unique_name.clone()),
            tier: item.tier,
            item_power: item.item_power,
            path: None, // carried over on merge
        });
    }
    rows
}

/// Builds consumable rows (adds `info`, taken from the dump for new items only).
pub fn consumable_rows_for(
    plan: &Plan,
    dataset: &Dataset,
    index: &IdentityIndex,
) -> Vec<ConsumableRow> {
    let mut rows = Vec::new();
    let mut next = next_id(&dataset.consumables, |r| r.id);
    for item in plan.items.iter().filter(|i| i.table == Table::Consumables) {
        let existing = index.find(&item.unique_name, &item.name);
        let id = existing.unwrap_or_else(|| {
            let id = next;
            next += 1;
            id
        });
        // `info` is only filled for brand-new consumables; the merge keeps the
        // wiki description for existing rows.
        let info = if existing.is_none() {
            item.description.clone()
        } else {
            None
        };
        rows.push(ConsumableRow {
            id,
            category_id: item.category_id,
            subcategory_id: item.subcategory_id,
            name: item.name.clone(),
            identifier: Some(item.unique_name.clone()),
            tier: item.tier,
            item_power: item.item_power,
            path: None,
            info,
        });
    }
    rows
}

/// Applies the plan to the dataset (overwrite semantics), returning the
/// per-table change statistics.
pub fn apply(plan: &Plan, dataset: &mut Dataset) -> HashMap<Table, OverwriteStats> {
    let index = dataset.identity_index();
    let mut stats: HashMap<Table, OverwriteStats> = HashMap::new();
    let excluded: std::collections::HashSet<&str> =
        plan.excluded_ids.iter().map(String::as_str).collect();

    for table in [Table::Weapons, Table::Armors, Table::Accessories] {
        let existing = dataset.items(table).to_vec();
        let incoming = rows_for(plan, table, dataset, &index);
        let (mut rows, mut item_stats) =
            crate::importer::store::merge_item_rows(&existing, incoming, &index);
        // Cosmetics the dump rejects must not linger from an earlier import.
        let before = rows.len();
        rows.retain(|row| !row.identifier.as_deref().is_some_and(|id| excluded.contains(id)));
        item_stats.removed += before - rows.len();
        // Pruned rows are not "kept because the dump lacks them".
        item_stats.carry_over_without_match = item_stats
            .carry_over_without_match
            .saturating_sub(item_stats.removed);
        if let Some(slot) = dataset.items_mut(table) {
            *slot = rows;
        }
        stats.insert(table, item_stats);
    }

    let existing = dataset.consumables.clone();
    let incoming = consumable_rows_for(plan, dataset, &index);
    let (mut rows, mut cons_stats) =
        crate::importer::store::merge_consumable_rows(&existing, incoming, &index);
    let before = rows.len();
    rows.retain(|row| !row.identifier.as_deref().is_some_and(|id| excluded.contains(id)));
    cons_stats.removed += before - rows.len();
    cons_stats.carry_over_without_match = cons_stats
        .carry_over_without_match
        .saturating_sub(cons_stats.removed);
    dataset.consumables = rows;
    stats.insert(Table::Consumables, cons_stats);

    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::importer::aod::AodItem;
    use crate::importer::store::CategoryRow;

    fn item(id: &str, name: &str, slot: &str, shop: &str, sub1: &str, tier: i64) -> AodItem {
        AodItem {
            unique_name: id.to_string(),
            name: name.to_string(),
            description: None,
            tier: Some(tier),
            item_power: None,
            slot_type: Some(slot.to_string()),
            shop_category: Some(shop.to_string()),
            shop_subcategory1: Some(sub1.to_string()),
            shop_subcategory2: None,
            mount_category: None,
            recipes: Vec::new(),
        }
    }

    fn dataset() -> Dataset {
        Dataset {
            categories: vec![
                CategoryRow {
                    id: 1,
                    name: "Warrior Weapons".into(),
                    parent_id: None,
                    path: None,
                    item_type: "weapon".into(),
                    version: 1,
                },
                CategoryRow {
                    id: 2,
                    name: "Axe".into(),
                    parent_id: Some(1),
                    path: Some("/wiki/Axe".into()),
                    item_type: "weapon".into(),
                    version: 1,
                },
            ],
            weapons: vec![ItemRow {
                id: 5,
                category_id: Some(1),
                subcategory_id: Some(2),
                name: "Adept's Battleaxe".into(),
                identifier: Some("T4_MAIN_AXE".into()),
                tier: Some(4.0),
                item_power: Some(700),
                path: Some("/wiki/Adept's_Battleaxe".into()),
            }],
            ..Dataset::default()
        }
    }

    fn dump(items: Vec<AodItem>) -> AodDump {
        let mut map = HashMap::new();
        for item in items {
            map.insert(item.unique_name.clone(), item);
        }
        AodDump { items: map }
    }

    #[test]
    fn keeps_existing_ids_and_adds_new_rows() {
        let mut ds = dataset();
        let d = dump(vec![
            item(
                "T4_MAIN_AXE",
                "Adept's Battleaxe",
                "mainhand",
                "weapons",
                "axe",
                4,
            ),
            item(
                "T5_MAIN_AXE",
                "Expert's Battleaxe",
                "mainhand",
                "weapons",
                "axe",
                5,
            ),
        ]);
        let plan = plan(&d, &ds, false);
        let stats = apply(&plan, &mut ds);
        assert_eq!(ds.weapons.len(), 2);
        // Existing row keeps id 5 and its wiki path.
        let old = ds
            .weapons
            .iter()
            .find(|r| r.identifier.as_deref() == Some("T4_MAIN_AXE"))
            .unwrap();
        assert_eq!(old.id, 5);
        assert_eq!(old.path.as_deref(), Some("/wiki/Adept's_Battleaxe"));
        // New row gets the next id and mapped category/subcategory.
        let new = ds
            .weapons
            .iter()
            .find(|r| r.identifier.as_deref() == Some("T5_MAIN_AXE"))
            .unwrap();
        assert_eq!(new.id, 6);
        assert_eq!(new.category_id, Some(1));
        assert_eq!(new.subcategory_id, Some(2));
        assert_eq!(stats[&Table::Weapons].inserted, 1);
    }

    #[test]
    fn learns_item_power_from_the_dataset_convention() {
        let ds = dataset();
        // The dump has no @itempower for the new T4 axe.
        let d = dump(vec![item(
            "T9_MAIN_AXE",
            "Skystrider Axe",
            "mainhand",
            "weapons",
            "axe",
            4,
        )]);
        let plan = plan(&d, &ds, false);
        let planned = plan
            .items
            .iter()
            .find(|i| i.unique_name == "T9_MAIN_AXE")
            .unwrap();
        // T4 weapons in the dataset are 700, so the convention wins.
        assert_eq!(planned.item_power, Some(700));
    }

    #[test]
    fn skips_cosmetics_unless_included() {
        let ds = dataset();
        let d = dump(vec![
            item(
                "UNIQUE_HEAD_VANITY_WARRIOR_KING",
                "Paragon's Crown",
                "head",
                "head",
                "other",
                4,
            ),
            item(
                "T5_MAIN_AXE",
                "Expert's Battleaxe",
                "mainhand",
                "weapons",
                "axe",
                5,
            ),
        ]);
        let without = plan(&d, &ds, false);
        assert_eq!(without.items.len(), 1);
        assert_eq!(without.skipped.len(), 1);
        let with = plan(&d, &ds, true);
        assert_eq!(with.items.len(), 2);
    }

    #[test]
    fn unmapped_subcategory_is_reported_not_invented() {
        let ds = dataset();
        // Gathering armor: classified as armor but no wiki subcategory.
        let d = dump(vec![item(
            "T4_ARMOR_GATHERER_FIBER",
            "Adept's Fiber Harvester",
            "armor",
            "gathering",
            "fiber",
            4,
        )]);
        let plan = plan(&d, &ds, false);
        assert_eq!(plan.items.len(), 1);
        assert!(plan.items[0].unmapped_subcategory);
        assert_eq!(plan.items[0].subcategory_id, None);
    }
}
