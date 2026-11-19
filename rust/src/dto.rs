//! Response DTOs mirroring Laravel's JsonResource classes.
//!
//! Struct field order == JSON key order (serde preserves declaration order),
//! which is exactly the order the PHP resource classes define their arrays.
//! `skip_serializing_if` replicates Laravel's `MissingValue` omission for
//! `whenLoaded` / `when($request->weaponry, ...)` fields.

use serde::Serialize;

use crate::models::{Category, Consumable, CraftingRow, Item, JsonText, Spell, StatRow};
use crate::render;
use crate::util::{plural_type, slot_name, strip_tags, tier_string};

// ---------------------------------------------------------------------------
// Generic wrappers
// ---------------------------------------------------------------------------

/// Laravel's resource-collection wrapper: `{"data": [...]}`.
#[derive(Serialize)]
pub struct DataList<T: Serialize> {
    pub data: Vec<T>,
}

impl<T: Serialize> DataList<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self { data }
    }
}

#[derive(Serialize)]
pub struct MessageDto {
    pub message: String,
}

impl MessageDto {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

// ---------------------------------------------------------------------------
// Categories (CategoryResource)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct CategoryDto {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    /// Only emitted on weaponry routes (`when($request->weaponry, ...)`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Only emitted when the children relation is loaded (root categories).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcategories: Option<Vec<CategoryDto>>,
}

impl CategoryDto {
    /// Category without children (as nested inside items, or as a subcategory
    /// listed under a root category).
    pub fn simple(category: &Category, weaponry: bool) -> Self {
        Self {
            id: category.id,
            name: category.name.clone(),
            item_type: category.item_type.clone(),
            path: weaponry.then(|| plural_type(&category.item_type)),
            subcategories: None,
        }
    }

    /// Root category with its children attached.
    pub fn root(category: &Category, children: Vec<CategoryDto>, weaponry: bool) -> Self {
        Self {
            path: weaponry.then(|| plural_type(&category.item_type)),
            subcategories: Some(children),
            ..Self::simple(category, weaponry)
        }
    }
}

// ---------------------------------------------------------------------------
// Items (WeaponResource / ArmorResource / AccessoryResource)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ItemDto {
    pub id: i64,
    pub name: String,
    pub tier: Option<String>,
    pub item_power: Option<i64>,
    pub identifier: String,
    pub icon: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<CategoryDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcategory: Option<CategoryDto>,
}

impl ItemDto {
    /// V1/V2 shape (no category relations loaded), also used for the `weapon`
    /// / `armor` / `accessory` object nested inside stat rows.
    pub fn flat(item: &Item) -> Self {
        Self {
            id: item.id,
            name: item.name.clone(),
            tier: tier_string(item.tier),
            item_power: item.item_power,
            identifier: item.identifier.clone(),
            icon: render::render_item(&item.identifier),
            category: None,
            subcategory: None,
        }
    }

    /// V3 shape (category + subcategory nested, no subcategories inside
    /// those nested objects).
    pub fn nested(item: &Item, category: Option<&Category>, subcategory: Option<&Category>) -> Self {
        Self {
            category: category.map(|c| CategoryDto::simple(c, false)),
            subcategory: subcategory.map(|c| CategoryDto::simple(c, false)),
            ..Self::flat(item)
        }
    }
}

// ---------------------------------------------------------------------------
// Consumables (ConsumableResource)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ConsumableDto {
    pub id: i64,
    pub name: String,
    pub tier: Option<String>,
    pub item_power: Option<i64>,
    pub identifier: String,
    pub icon: String,
    pub info: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<CategoryDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcategory: Option<CategoryDto>,
}

impl ConsumableDto {
    pub fn flat(consumable: &Consumable) -> Self {
        Self {
            id: consumable.id,
            name: consumable.name.clone(),
            tier: tier_string(consumable.tier),
            item_power: consumable.item_power,
            identifier: consumable.identifier.clone(),
            icon: render::render_item(&consumable.identifier),
            info: consumable.info.clone(),
            category: None,
            subcategory: None,
        }
    }

    pub fn nested(consumable: &Consumable, category: Option<&Category>, subcategory: Option<&Category>) -> Self {
        Self {
            category: category.map(|c| CategoryDto::simple(c, false)),
            subcategory: subcategory.map(|c| CategoryDto::simple(c, false)),
            ..Self::flat(consumable)
        }
    }
}

// ---------------------------------------------------------------------------
// Spells (SpellResource + grouped slots)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SpellDto {
    pub id: i64,
    pub name: String,
    pub slot: String,
    pub preview: Option<String>,
    pub icon: String,
    pub attributes: JsonText,
    pub description: Option<String>,
    pub description_html: Option<String>,
}

impl SpellDto {
    pub fn new(spell: &Spell) -> Self {
        Self {
            id: spell.id,
            name: spell.name.clone(),
            slot: slot_name(&spell.slot),
            preview: spell.preview.clone(),
            icon: render::render_spell(&spell.identifier),
            attributes: spell.attributes.clone(),
            description: spell.description.as_deref().map(strip_tags),
            description_html: spell.description.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct SlotGroupDto {
    pub slot: String,
    pub spells: Vec<SpellDto>,
}

// ---------------------------------------------------------------------------
// Stats (*StatResource + grouped by enchantment)
// ---------------------------------------------------------------------------

macro_rules! stat_row_dto {
    ($name:ident, $key:literal) => {
        #[derive(Serialize)]
        pub struct $name {
            pub id: i64,
            pub quality: String,
            pub enchantment: i64,
            #[serde(rename = $key)]
            pub item: Option<ItemDto>,
            pub stats: JsonText,
        }

        impl $name {
            pub fn new(row: &StatRow, item: Option<ItemDto>) -> Self {
                Self {
                    id: row.id,
                    quality: row.quality.clone(),
                    enchantment: row.enchantment,
                    item,
                    stats: row.stats.clone(),
                }
            }
        }
    };
}

stat_row_dto!(WeaponStatRowDto, "weapon");
stat_row_dto!(ArmorStatRowDto, "armor");
stat_row_dto!(AccessoryStatRowDto, "accessory");

#[derive(Serialize)]
pub struct ConsumableStatRowDto {
    pub id: i64,
    pub quality: String,
    pub enchantment: i64,
    pub consumable: Option<ConsumableDto>,
    pub stats: JsonText,
}

impl ConsumableStatRowDto {
    pub fn new(row: &StatRow, consumable: Option<ConsumableDto>) -> Self {
        Self {
            id: row.id,
            quality: row.quality.clone(),
            enchantment: row.enchantment,
            consumable,
            stats: row.stats.clone(),
        }
    }
}

macro_rules! stat_group_dto {
    ($name:ident, $row:ty) => {
        #[derive(Serialize)]
        pub struct $name {
            pub enchantment: i64,
            pub icon: String,
            pub stats: Vec<$row>,
        }
    };
}

stat_group_dto!(WeaponStatGroupDto, WeaponStatRowDto);
stat_group_dto!(ArmorStatGroupDto, ArmorStatRowDto);
stat_group_dto!(AccessoryStatGroupDto, AccessoryStatRowDto);
stat_group_dto!(ConsumableStatGroupDto, ConsumableStatRowDto);

// ---------------------------------------------------------------------------
// Consumable craftings (ConsumableCraftingResource + grouped)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct CraftingRowDto {
    pub id: i64,
    pub per_craft: Option<i64>,
    pub enchantment: i64,
    pub consumable: Option<ConsumableDto>,
    pub requirements: JsonText,
}

impl CraftingRowDto {
    pub fn new(row: &CraftingRow, consumable: Option<ConsumableDto>) -> Self {
        Self {
            id: row.id,
            per_craft: row.per_craft,
            enchantment: row.enchantment,
            consumable,
            requirements: row.requirements.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct CraftingGroupDto {
    pub enchantment: i64,
    pub icon: String,
    pub crafting: CraftingRowDto,
}

// ---------------------------------------------------------------------------
// Weaponry search (SearchResource)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SearchItemDto {
    #[serde(rename = "type")]
    pub item_type: String,
    pub type_id: i64,
    pub name: String,
    pub tier: Option<String>,
    pub item_power: Option<i64>,
    pub identifier: String,
    pub info: Option<String>,
    pub icon: String,
}

impl SearchItemDto {
    pub fn weapon(item: &Item) -> Self {
        Self::new("weapon", item.id, &item.name, item.tier, item.item_power, &item.identifier, None)
    }

    pub fn armor(item: &Item) -> Self {
        Self::new("armor", item.id, &item.name, item.tier, item.item_power, &item.identifier, None)
    }

    pub fn accessory(item: &Item) -> Self {
        Self::new("accessory", item.id, &item.name, item.tier, item.item_power, &item.identifier, None)
    }

    pub fn consumable(item: &Consumable) -> Self {
        Self::new(
            "consumable",
            item.id,
            &item.name,
            item.tier,
            item.item_power,
            &item.identifier,
            item.info.clone(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        item_type: &str,
        type_id: i64,
        name: &str,
        tier: Option<f64>,
        item_power: Option<i64>,
        identifier: &str,
        info: Option<String>,
    ) -> Self {
        Self {
            item_type: item_type.to_string(),
            type_id,
            name: name.to_string(),
            tier: tier_string(tier),
            item_power,
            identifier: identifier.to_string(),
            info,
            icon: render::render_item(identifier),
        }
    }
}

// ---------------------------------------------------------------------------
// Misc
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct VersionCheckDto {
    pub title: &'static str,
    pub description: &'static str,
    pub version: &'static str,
    pub force: bool,
}

impl VersionCheckDto {
    pub fn latest() -> Self {
        Self {
            title: "Update Available!",
            description: "[+] Add Food & Potions\n[+] Add Crafting Process",
            version: "1.0.7",
            force: false,
        }
    }
}

/// Laravel-style 422 validation failure. Field order matches the rule
/// declaration order in `BugReportRequest`.
#[derive(Serialize)]
pub struct ValidationErrorDto {
    /// Laravel's summary: the first error message, plus
    /// " (and N more error[s])" when there are several.
    pub message: String,
    pub errors: ValidationErrorsDto,
}

impl ValidationErrorDto {
    /// Summarizes exactly like Laravel's ValidationException.
    pub fn summarize(errors: ValidationErrorsDto) -> Self {
        let all: Vec<String> = [errors.category.as_ref(), errors.description.as_ref(), errors.debug.as_ref()]
            .into_iter()
            .flatten()
            .flat_map(|v| v.iter().cloned())
            .collect();

        let mut message = all.first().cloned().unwrap_or_default();
        let extra = all.len().saturating_sub(1);
        if extra > 0 {
            let plural = if extra > 1 { "s" } else { "" };
            message.push_str(&format!(" (and {extra} more error{plural})"));
        }

        Self { message, errors }
    }
}

#[derive(Serialize)]
pub struct ValidationErrorsDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<Vec<String>>,
}
