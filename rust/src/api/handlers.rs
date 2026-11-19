//! Shared handlers for the v1 / v2 / v3 / weaponry(=v1) / weaponryV2(=v2)
//! route sets. Every handler is parameterized by [`Version`] (and the
//! weaponry flag where it matters), because the route *sets* are identical
//! and only the resource shapes differ:
//!
//! - v1 categories are filtered to `version <= 1`, v2/v3 to `version <= 2`;
//! - v1/v2 item resources are flat, v3 nests `category` + `subcategory`;
//! - `path` on categories is emitted only under the weaponry prefixes.

use axum::Json;
use serde::Deserialize;

use crate::dto::*;
use crate::models::{Category, Consumable, Item, Spell, StatRow};
use crate::state::AppState;
use crate::util::{php_query_float, php_query_number, php_query_string};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    V1,
    V2,
    V3,
}

impl Version {
    pub fn max_category_version(&self) -> i64 {
        match self {
            Version::V1 => 1,
            Version::V2 | Version::V3 => 2,
        }
    }

    /// v3 nests category/subcategory in item resources; v1/v2 do not.
    pub fn nested_items(&self) -> bool {
        matches!(self, Version::V3)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Weapon,
    Armor,
    Accessory,
    Consumable,
}

// ---------------------------------------------------------------------------
// Query types (raw strings so PHP falsy semantics can be reproduced)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CategoryQuery {
    pub r#type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ItemQuery {
    pub category_id: Option<String>,
    pub subcategory_id: Option<String>,
    pub tier: Option<String>,
}

/// Resolved filters for one request. `Err(())` means a truthy but
/// non-numeric value was supplied, which matches no rows (SQLite numeric
/// affinity comparison), so the whole list comes back empty.
#[derive(Debug)]
struct ItemFilters {
    category_id: Result<Option<i64>, ()>,
    subcategory_id: Result<Option<i64>, ()>,
    tier: Result<Option<f64>, ()>,
}

impl ItemFilters {
    fn from_query(q: &ItemQuery) -> Self {
        Self {
            category_id: php_query_number(&q.category_id),
            subcategory_id: php_query_number(&q.subcategory_id),
            tier: php_query_float(&q.tier),
        }
    }

    fn matches(&self, category_id: Option<i64>, subcategory_id: Option<i64>, tier: Option<f64>) -> bool {
        let (Ok(c), Ok(s), Ok(t)) = (&self.category_id, &self.subcategory_id, &self.tier) else {
            return false;
        };
        c.is_none_or(|n| category_id == Some(n))
            && s.is_none_or(|n| subcategory_id == Some(n))
            && t.is_none_or(|t| tier == Some(t))
    }
}

// ---------------------------------------------------------------------------
// Categories
// ---------------------------------------------------------------------------

pub fn categories_index(
    state: &AppState,
    version: Version,
    weaponry: bool,
    q: &CategoryQuery,
) -> Json<DataList<CategoryDto>> {
    let type_filter = php_query_string(&q.r#type);

    let data: Vec<CategoryDto> = state
        .categories
        .iter()
        .filter(|c| c.parent_id.is_none())
        .filter(|c| c.version <= version.max_category_version())
        .filter(|c| type_filter.as_ref().is_none_or(|t| &c.item_type == t))
        .map(|c| {
            let children: Vec<CategoryDto> = state
                .categories
                .iter()
                // Children load unconditionally (no version filter),
                // exactly like Laravel's `with('children')`.
                .filter(|child| child.parent_id == Some(c.id))
                .map(|child| CategoryDto::simple(child, weaponry))
                .collect();
            CategoryDto::root(c, children, weaponry)
        })
        .collect();

    Json(DataList::new(data))
}

// ---------------------------------------------------------------------------
// Item lists (weapons / armors / accessories / consumables)
// ---------------------------------------------------------------------------

fn nested_parts(
    state: &AppState,
    category_id: Option<i64>,
    subcategory_id: Option<i64>,
) -> (Option<&Category>, Option<&Category>) {
    (state.category(category_id), state.category(subcategory_id))
}

pub fn weapons_index(state: &AppState, version: Version, q: &ItemQuery) -> Json<DataList<ItemDto>> {
    let filters = ItemFilters::from_query(q);
    let data: Vec<ItemDto> = state
        .weapons
        .iter()
        .filter(|w| filters.matches(w.category_id, w.subcategory_id, w.tier))
        .map(|w| item_dto(state, version, w))
        .collect();
    Json(DataList::new(data))
}

pub fn armors_index(state: &AppState, version: Version, q: &ItemQuery) -> Json<DataList<ItemDto>> {
    let filters = ItemFilters::from_query(q);
    let data: Vec<ItemDto> = state
        .armors
        .iter()
        .filter(|a| filters.matches(a.category_id, a.subcategory_id, a.tier))
        .map(|a| item_dto(state, version, a))
        .collect();
    Json(DataList::new(data))
}

pub fn accessories_index(state: &AppState, version: Version, q: &ItemQuery) -> Json<DataList<ItemDto>> {
    let filters = ItemFilters::from_query(q);
    let data: Vec<ItemDto> = state
        .accessories
        .iter()
        .filter(|a| filters.matches(a.category_id, a.subcategory_id, a.tier))
        .map(|a| item_dto(state, version, a))
        .collect();
    Json(DataList::new(data))
}

pub fn consumables_index(state: &AppState, version: Version, q: &ItemQuery) -> Json<DataList<ConsumableDto>> {
    let filters = ItemFilters::from_query(q);
    let data: Vec<ConsumableDto> = state
        .consumables
        .iter()
        .filter(|c| filters.matches(c.category_id, c.subcategory_id, c.tier))
        .map(|c| {
            if version.nested_items() {
                let (category, subcategory) = nested_parts(state, c.category_id, c.subcategory_id);
                ConsumableDto::nested(c, category, subcategory)
            } else {
                ConsumableDto::flat(c)
            }
        })
        .collect();
    Json(DataList::new(data))
}

fn item_dto(state: &AppState, version: Version, item: &Item) -> ItemDto {
    if version.nested_items() {
        let (category, subcategory) = nested_parts(state, item.category_id, item.subcategory_id);
        ItemDto::nested(item, category, subcategory)
    } else {
        ItemDto::flat(item)
    }
}

// ---------------------------------------------------------------------------
// Spells by item id (grouped by slot)
// ---------------------------------------------------------------------------

/// Parses the `{weaponId}` path param; a non-numeric id yields an empty
/// result (Laravel would trip over the SQL comparison — documented
/// deviation).
fn parse_path_id(raw: &str) -> Option<i64> {
    raw.parse::<i64>().ok()
}

/// Spells belonging to an item, in spells-table (rowid) order — mirrors
/// `Spell::whereHas('weaponSpells', ...)->get()`, which orders by the
/// spells table, not the pivot.
fn spells_for(state: &AppState, kind: ItemKind, item_id: i64) -> Vec<&Spell> {
    let ids = match kind {
        ItemKind::Weapon => state.weapon_spells.get(&item_id),
        ItemKind::Armor => state.armor_spells.get(&item_id),
        ItemKind::Accessory => state.accessory_spells.get(&item_id),
        ItemKind::Consumable => None,
    };
    match ids {
        Some(ids) if !ids.is_empty() => {
            state.spells.iter().filter(|s| ids.contains(&s.id)).collect()
        }
        _ => Vec::new(),
    }
}

pub fn spells_by_item(state: &AppState, kind: ItemKind, raw_id: &str) -> Json<DataList<SlotGroupDto>> {
    let spells = parse_path_id(raw_id)
        .map(|id| spells_for(state, kind, id))
        .unwrap_or_default();

    // Group by slot display name, first-seen order (Laravel's groupBy).
    let mut groups: Vec<SlotGroupDto> = Vec::new();
    for spell in spells {
        let slot = crate::util::slot_name(&spell.slot);
        match groups.iter_mut().find(|g| g.slot == slot) {
            Some(group) => group.spells.push(SpellDto::new(spell)),
            None => groups.push(SlotGroupDto {
                slot,
                spells: vec![SpellDto::new(spell)],
            }),
        }
    }

    Json(DataList::new(groups))
}

// ---------------------------------------------------------------------------
// Stats by item id (grouped by enchantment)
// ---------------------------------------------------------------------------

fn stats_rows(state: &AppState, kind: ItemKind, item_id: i64) -> &[StatRow] {
    let rows = match kind {
        ItemKind::Weapon => state.weapon_stats.get(&item_id),
        ItemKind::Armor => state.armor_stats.get(&item_id),
        ItemKind::Accessory => state.accessory_stats.get(&item_id),
        ItemKind::Consumable => state.consumable_stats.get(&item_id),
    };
    rows.map(Vec::as_slice).unwrap_or_default()
}

/// Group boundaries in first-seen order: `(enchantment, indices of rows)`.
fn enchantment_groups(rows: &[StatRow]) -> Vec<(i64, Vec<usize>)> {
    let mut groups: Vec<(i64, Vec<usize>)> = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        match groups.iter_mut().find(|(e, _)| *e == row.enchantment) {
            Some((_, idxs)) => idxs.push(i),
            None => groups.push((row.enchantment, vec![i])),
        }
    }
    groups
}

macro_rules! plain_stats_by {
    ($fn_name:ident, $kind:expr, $index:expr, $row_dto:ident, $group_dto:ident) => {
        pub fn $fn_name(state: &AppState, raw_id: &str) -> Json<DataList<$group_dto>> {
            let Some(item_id) = parse_path_id(raw_id) else {
                return Json(DataList::new(Vec::new()));
            };
            let rows = stats_rows(state, $kind, item_id);
            let groups = enchantment_groups(rows);

            let data: Vec<$group_dto> = groups
                .into_iter()
                .map(|(enchantment, idxs)| {
                    let stat_dtos: Vec<$row_dto> = idxs
                        .iter()
                        .map(|&i| {
                            let row = &rows[i];
                            let item = $index(state, row.item_id);
                            $row_dto::new(row, item.map(ItemDto::flat))
                        })
                        .collect();
                    // Icon comes from the first row's item, enchanted.
                    let icon = idxs
                        .first()
                        .and_then(|&i| $index(state, rows[i].item_id))
                        .map(|item| {
                            crate::render::render_item_enchanted(&item.identifier, enchantment)
                        })
                        .unwrap_or_default();
                    $group_dto { enchantment, icon, stats: stat_dtos }
                })
                .collect();

            Json(DataList::new(data))
        }
    };
}

fn weapon_by_id(state: &AppState, id: i64) -> Option<&Item> {
    state.weapon_index.get(&id).copied().map(|i| &state.weapons[i])
}

fn armor_by_id(state: &AppState, id: i64) -> Option<&Item> {
    state.armor_index.get(&id).copied().map(|i| &state.armors[i])
}

fn accessory_by_id(state: &AppState, id: i64) -> Option<&Item> {
    state.accessory_index.get(&id).copied().map(|i| &state.accessories[i])
}

plain_stats_by!(weapon_stats_by, ItemKind::Weapon, weapon_by_id, WeaponStatRowDto, WeaponStatGroupDto);
plain_stats_by!(armor_stats_by, ItemKind::Armor, armor_by_id, ArmorStatRowDto, ArmorStatGroupDto);
plain_stats_by!(
    accessory_stats_by,
    ItemKind::Accessory,
    accessory_by_id,
    AccessoryStatRowDto,
    AccessoryStatGroupDto
);

pub fn consumable_stats_by(state: &AppState, raw_id: &str) -> Json<DataList<ConsumableStatGroupDto>> {
    let Some(item_id) = parse_path_id(raw_id) else {
        return Json(DataList::new(Vec::new()));
    };
    let rows = stats_rows(state, ItemKind::Consumable, item_id);
    let groups = enchantment_groups(rows);

    let consumable_by_id = |id: i64| -> Option<&Consumable> {
        state.consumable_index.get(&id).copied().map(|i| &state.consumables[i])
    };

    let data: Vec<ConsumableStatGroupDto> = groups
        .into_iter()
        .map(|(enchantment, idxs)| {
            let stat_dtos: Vec<ConsumableStatRowDto> = idxs
                .iter()
                .map(|&i| {
                    let row = &rows[i];
                    let consumable = consumable_by_id(row.item_id);
                    ConsumableStatRowDto::new(row, consumable.map(ConsumableDto::flat))
                })
                .collect();
            let icon = idxs
                .first()
                .and_then(|&i| consumable_by_id(rows[i].item_id))
                .map(|c| crate::render::render_item_enchanted(&c.identifier, enchantment))
                .unwrap_or_default();
            ConsumableStatGroupDto { enchantment, icon, stats: stat_dtos }
        })
        .collect();

    Json(DataList::new(data))
}

// ---------------------------------------------------------------------------
// Consumable craftings by consumable id
// ---------------------------------------------------------------------------

pub fn craftings_by(state: &AppState, raw_id: &str) -> Json<DataList<CraftingGroupDto>> {
    let Some(item_id) = parse_path_id(raw_id) else {
        return Json(DataList::new(Vec::new()));
    };
    let rows = state.consumable_craftings.get(&item_id).map(Vec::as_slice).unwrap_or_default();

    // Group by enchantment, first-seen order; the group payload is the
    // first row of each group (Laravel's `$group->first()`).
    let mut groups: Vec<(i64, Vec<usize>)> = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        match groups.iter_mut().find(|(e, _)| *e == row.enchantment) {
            Some((_, idxs)) => idxs.push(i),
            None => groups.push((row.enchantment, vec![i])),
        }
    }

    let consumable_by_id = |id: i64| -> Option<&Consumable> {
        state.consumable_index.get(&id).copied().map(|i| &state.consumables[i])
    };

    let data: Vec<CraftingGroupDto> = groups
        .into_iter()
        .map(|(enchantment, idxs)| {
            let first = idxs
                .first()
                .map(|&i| &rows[i])
                .expect("enchantment group is never empty");
            let consumable = consumable_by_id(first.item_id);
            let crafting = CraftingRowDto::new(first, consumable.map(ConsumableDto::flat));
            let icon = consumable
                .map(|c| crate::render::render_item_enchanted(&c.identifier, enchantment))
                .unwrap_or_default();
            CraftingGroupDto { enchantment, icon, crafting }
        })
        .collect();

    Json(DataList::new(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_follow_php_falsy_semantics() {
        let q = ItemQuery {
            category_id: Some("1".into()),
            subcategory_id: Some("0".into()), // falsy -> no filter
            tier: None,
        };
        let f = ItemFilters::from_query(&q);
        assert!(f.matches(Some(1), Some(2), None));
        assert!(!f.matches(Some(2), Some(2), None));

        let q = ItemQuery {
            category_id: Some("abc".into()),
            subcategory_id: None,
            tier: None,
        };
        let f = ItemFilters::from_query(&q);
        assert!(!f.matches(Some(1), Some(2), Some(4.0))); // matches nothing

        let q = ItemQuery { category_id: None, subcategory_id: None, tier: Some("4".into()) };
        let f = ItemFilters::from_query(&q);
        assert!(f.matches(Some(1), Some(2), Some(4.0)));
        assert!(!f.matches(Some(1), Some(2), Some(5.0)));
    }
}
