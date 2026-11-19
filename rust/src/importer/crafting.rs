//! Regenerating consumable crafting recipes from the dump.
//!
//! The dump carries, per consumable, a base `craftingrequirements` plus an
//! `enchantments.enchantment[]` list with one recipe per enchantment level.
//! That maps 1:1 onto our `consumable_craftings` table:
//!
//! | dataset                       | dump                                  |
//! |-------------------------------|---------------------------------------|
//! | `enchantment`                 | 0 for the base recipe, level otherwise |
//! | `per_craft`                   | `@amountcrafted`                       |
//! | `requirements[].value`        | `craftresource[].@count`               |
//! | `requirements[].identifier`   | `craftresource[].@uniquename`          |
//! | `requirements[].name`         | dump English name of the resource      |
//! | `requirements[].icon`         | `render.albiononline.com` item URL     |

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::importer::aod::AodDump;
use crate::importer::store::{next_id, normalize_name};
use crate::render;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingRow {
    pub id: i64,
    pub consumable_id: i64,
    pub per_craft: Option<i64>,
    pub enchantment: i64,
    pub requirements: Value,
}

/// Builds the requirement array for one recipe the way the dataset stores it
/// (key order matches the SQLite jsonb export: icon, identifier, name, value).
pub fn requirements_for(dump: &AodDump, craft: &crate::importer::aod::Recipe) -> Value {
    let mut out: Vec<Value> = Vec::new();
    for resource in &craft.resources {
        if resource.count <= 0 {
            continue;
        }
        let name = dump
            .get(&resource.unique_name)
            .map(|r| r.name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| resource.unique_name.clone());
        out.push(json!({
            "icon": render::render_item(&resource.unique_name),
            "identifier": resource.unique_name,
            "name": name,
            "value": resource.count,
        }));
    }
    Value::Array(out)
}

/// How many crafting rows were added / updated / left alone.
#[derive(Debug, Default)]
pub struct CraftingStats {
    pub inserted: usize,
    pub updated: usize,
    pub unchanged: usize,
    /// Dataset rows with no counterpart in the dump (kept, reported).
    pub kept_without_match: usize,
    pub by_consumable: usize,
}

impl CraftingStats {
    pub fn changed(&self) -> bool {
        self.inserted > 0 || self.updated > 0
    }
}

/// Regenerates the crafting rows for every consumable that exists in both the
/// dataset and the dump. Existing rows are matched by
/// `(consumable_id, per_craft, enchantment)` — the same key the Laravel
/// `UpdateConsumableCrafting` action used — so ids stay stable.
pub fn regenerate(
    dump: &AodDump,
    existing: &[CraftingRow],
    consumables: &[(i64, String, String)], // (id, identifier, name)
    next_missing_id: i64,
) -> (Vec<CraftingRow>, CraftingStats) {
    let mut stats = CraftingStats::default();
    let mut rows: Vec<CraftingRow> = existing.to_vec();
    let mut next_id = next_missing_id;

    // case-insensitive name -> unique name, to match consumables whose dataset
    // identifier is a mangled name (e.g. the Focus potion).
    let mut by_name = std::collections::HashMap::new();
    for item in dump.sorted() {
        if !item.name.is_empty() {
            by_name
                .entry(normalize_name(&item.name))
                .or_insert(item.unique_name.clone());
        }
    }

    for (consumable_id, identifier, name) in consumables {
        let Some(unique_name) = dump
            .get(identifier)
            .map(|i| i.unique_name.clone())
            .or_else(|| by_name.get(&normalize_name(name)).cloned())
        else {
            continue;
        };
        let Some(item) = dump.get(&unique_name) else {
            continue;
        };
        stats.by_consumable += 1;

        for (level, recipe) in item.recipes.iter().enumerate() {
            if recipe.resources.is_empty() {
                continue;
            }
            let enchantment = level as i64;
            let requirements = requirements_for(dump, recipe);
            let per_craft = recipe.amount_crafted;
            // Match on the historical key first, then on enchantment alone
            // (the dump's `amountcrafted` may legitimately change).
            let position = rows.iter().position(|row| {
                row.consumable_id == *consumable_id
                    && row.enchantment == enchantment
                    && row.per_craft == per_craft
            });
            let fallback = rows.iter().position(|row| {
                row.consumable_id == *consumable_id && row.enchantment == enchantment
            });
            match position.or(fallback) {
                Some(index) => {
                    let row = &mut rows[index];
                    if row.per_craft == per_craft && row.requirements == requirements {
                        stats.unchanged += 1;
                    } else {
                        row.per_craft = per_craft;
                        row.requirements = requirements;
                        stats.updated += 1;
                    }
                }
                None => {
                    rows.push(CraftingRow {
                        id: next_id,
                        consumable_id: *consumable_id,
                        per_craft,
                        enchantment,
                        requirements,
                    });
                    next_id += 1;
                    stats.inserted += 1;
                }
            }
        }
    }

    // Count dataset rows that the dump could not describe (never deleted).
    let covered: std::collections::HashSet<i64> = consumables
        .iter()
        .filter(|(_, identifier, name)| {
            dump.get(identifier).is_some() || by_name.contains_key(&normalize_name(name))
        })
        .map(|(id, _, _)| *id)
        .collect();
    stats.kept_without_match = existing
        .iter()
        .filter(|row| !covered.contains(&row.consumable_id))
        .count();

    (rows, stats)
}

/// Convenience for callers building a fresh id counter.
pub fn next_crafting_id(rows: &[CraftingRow]) -> i64 {
    next_id(rows, |row| row.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::importer::aod::{AodDump, AodItem, Recipe, Resource};
    use std::collections::HashMap;

    fn dump_with(item: AodItem) -> AodDump {
        let mut items = HashMap::new();
        items.insert(item.unique_name.clone(), item);
        // A resource item so names resolve.
        items.insert(
            "T4_BURDOCK".to_string(),
            AodItem {
                unique_name: "T4_BURDOCK".to_string(),
                name: "Burdock".to_string(),
                description: None,
                tier: Some(4),
                item_power: None,
                slot_type: None,
                shop_category: None,
                shop_subcategory1: None,
                shop_subcategory2: None,
                mount_category: None,
                recipes: Vec::new(),
            },
        );
        AodDump { items }
    }

    fn potion() -> AodItem {
        AodItem {
            unique_name: "T4_POTION_HEAL".to_string(),
            name: "Healing Potion".to_string(),
            description: None,
            tier: Some(4),
            item_power: None,
            slot_type: Some("potion".to_string()),
            shop_category: Some("consumables".to_string()),
            shop_subcategory1: Some("potions".to_string()),
            shop_subcategory2: Some("heal".to_string()),
            mount_category: None,
            recipes: vec![
                Recipe {
                    amount_crafted: Some(5),
                    resources: vec![Resource {
                        unique_name: "T4_BURDOCK".to_string(),
                        count: 24,
                    }],
                },
                Recipe {
                    amount_crafted: Some(5),
                    resources: vec![
                        Resource {
                            unique_name: "T4_BURDOCK".to_string(),
                            count: 24,
                        },
                        Resource {
                            unique_name: "T4_RUNE".to_string(),
                            count: 8,
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn builds_requirements_in_dataset_key_order() {
        let dump = dump_with(potion());
        let item = dump.get("T4_POTION_HEAL").unwrap();
        let reqs = requirements_for(&dump, &item.recipes[0]);
        let first = &reqs[0];
        assert_eq!(first["identifier"], "T4_BURDOCK");
        assert_eq!(first["name"], "Burdock");
        assert_eq!(first["value"], 24);
        assert!(first["icon"]
            .as_str()
            .unwrap()
            .starts_with("https://render.albiononline.com/v1/item/T4_BURDOCK@0.png"));
        // serialized key order follows the JSON object literal order
        let text = serde_json::to_string(first).unwrap();
        let positions: Vec<usize> = ["\"icon\"", "\"identifier\"", "\"name\"", "\"value\""]
            .iter()
            .map(|k| text.find(k).unwrap())
            .collect();
        assert!(positions.windows(2).all(|w| w[0] < w[1]), "{text}");
    }

    #[test]
    fn inserts_missing_enchantment_rows_with_stable_ids() {
        let dump = dump_with(potion());
        let existing = vec![CraftingRow {
            id: 10,
            consumable_id: 1,
            per_craft: Some(5),
            enchantment: 0,
            requirements: Value::Array(Vec::new()),
        }];
        let (rows, stats) = regenerate(
            &dump,
            &existing,
            &[(
                1,
                "T4_POTION_HEAL".to_string(),
                "Healing Potion".to_string(),
            )],
            11,
        );
        assert_eq!(rows.len(), 2);
        // Existing row updated in place, new one gets the next id.
        assert_eq!(rows[0].id, 10);
        assert_eq!(rows[1].id, 11);
        assert_eq!(rows[1].enchantment, 1);
        assert_eq!(stats.updated, 1);
        assert_eq!(stats.inserted, 1);
        assert_eq!(stats.kept_without_match, 0);
    }

    #[test]
    fn matches_by_name_when_identifier_is_a_mangled_name() {
        let dump = dump_with(potion());
        // Dataset row whose identifier is the display name (Focus-potion case).
        let (rows, stats) = regenerate(
            &dump,
            &[],
            &[(
                7,
                "Healing Potion".to_string(),
                "Healing Potion".to_string(),
            )],
            1,
        );
        assert_eq!(stats.inserted, 2, "name fallback matches the potion");
        assert!(rows.iter().all(|r| r.consumable_id == 7));
    }

    #[test]
    fn rows_without_a_dump_counterpart_are_kept() {
        let dump = dump_with(potion());
        let existing = vec![CraftingRow {
            id: 3,
            consumable_id: 99, // not in the consumable list
            per_craft: Some(1),
            enchantment: 0,
            requirements: Value::Array(Vec::new()),
        }];
        let (rows, stats) = regenerate(&dump, &existing, &[], 4);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, 3);
        assert_eq!(stats.kept_without_match, 1);
    }

    #[test]
    fn per_craft_change_updates_the_existing_row() {
        let dump = dump_with(potion());
        let existing = vec![CraftingRow {
            id: 10,
            consumable_id: 1,
            per_craft: Some(1), // dump says 5
            enchantment: 0,
            requirements: Value::Array(Vec::new()),
        }];
        let (rows, stats) = regenerate(
            &dump,
            &existing,
            &[(
                1,
                "T4_POTION_HEAL".to_string(),
                "Healing Potion".to_string(),
            )],
            11,
        );
        assert_eq!(rows[0].id, 10);
        assert_eq!(rows[0].per_craft, Some(5));
        assert_eq!(stats.updated, 1);
        assert_eq!(stats.inserted, 1);
    }
}
