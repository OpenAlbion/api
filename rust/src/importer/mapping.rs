//! Classification and category mapping.
//!
//! AOD describes items with `shop_category` / `shop_subcategory1/2` /
//! `slot_type` / `mount_category`; the dataset uses the wiki's category tree
//! (`categories.json`). The mapping below was derived from the intersection of
//! both sources (every existing item matches a rule) and is frozen here so
//! imports are deterministic.

use crate::importer::aod::AodItem;

/// Which dataset table an AOD item belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Table {
    Weapons,
    Armors,
    Accessories,
    Consumables,
}

impl Table {
    pub fn file(self) -> &'static str {
        match self {
            Table::Weapons => "weapons.json",
            Table::Armors => "armors.json",
            Table::Accessories => "accessories.json",
            Table::Consumables => "consumables.json",
        }
    }

    pub fn label(self) -> &'static str {
        self.file().trim_end_matches(".json")
    }
}

/// Where an item sits in the category tree, by category/subcategory *name*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryPath {
    pub category: String,
    pub subcategory: Option<String>,
}

impl CategoryPath {
    fn new(category: &str, subcategory: Option<&str>) -> Self {
        Self {
            category: category.to_string(),
            subcategory: subcategory.map(str::to_string),
        }
    }
}

/// Classifies an item into a dataset table, or `None` when it is not a
/// gameplay item we import.
pub fn table_of(item: &AodItem) -> Option<Table> {
    let slot = item.slot_type.as_deref();
    let shop = item.shop_category.as_deref();
    let sub1 = item.shop_subcategory1.as_deref();

    match (slot, shop) {
        (Some("mainhand"), Some("weapons")) => Some(Table::Weapons),
        (Some("offhand"), Some("offhands")) => Some(Table::Weapons),
        (Some("armor" | "head" | "shoes"), Some("armors" | "head" | "shoes")) => {
            Some(Table::Armors)
        }
        (Some("cape"), Some("capes")) => Some(Table::Accessories),
        (Some("bag"), Some("bags")) => Some(Table::Accessories),
        (Some("mount"), Some("mounts")) => Some(Table::Accessories),
        (Some("food"), Some("consumables")) => Some(Table::Consumables),
        (Some("potion"), Some("consumables")) => Some(Table::Consumables),
        // Gathering gear and cosmetics live under an "other" shop category
        // but are still real armor pieces.
        (Some("armor" | "head" | "shoes"), Some("gathering" | "other")) => Some(Table::Armors),
        (Some("cape" | "bag"), Some("gathering" | "other" | "vanity")) => {
            Some(Table::Accessories)
        }
        _ => {
            // Fall back on the weapon section markers: AOD marks off-hands and
            // shapeshifter staves with varying shop categories.
            match sub1 {
                Some("shieldtype" | "torchtype" | "booktype") => Some(Table::Weapons),
                _ => None,
            }
        }
    }
}

/// Maps an item to its dataset category/subcategory names.
///
/// Returns `None` for the subcategory when the wiki tree has no matching
/// entry (the parent category is still returned): the store then writes
/// `subcategory_id = null`, and the report lists the item.
pub fn category_path(item: &AodItem) -> Option<CategoryPath> {
    match table_of(item)? {
        Table::Weapons => Some(weapon_path(item)),
        Table::Armors => Some(armor_path(item)),
        Table::Accessories => Some(accessory_path(item)),
        Table::Consumables => Some(consumable_path(item)),
    }
}

// ---------------------------------------------------------------------------
// Weapons: (shop_subcategory1, slot_type) -> (category, subcategory)
// ---------------------------------------------------------------------------

fn weapon_path(item: &AodItem) -> CategoryPath {
    let sub1 = item.shop_subcategory1.as_deref().unwrap_or("");
    let (category, subcategory) = match sub1 {
        "sword" => ("Warrior Weapons", "Sword"),
        "axe" => ("Warrior Weapons", "Axe"),
        "mace" => ("Warrior Weapons", "Mace"),
        "hammer" => ("Warrior Weapons", "Hammer"),
        "crossbow" => ("Warrior Weapons", "Crossbow"),
        "knuckles" => ("Warrior Weapons", "War Gloves"),
        "shieldtype" => ("Warrior Weapons", "Shield"),
        "bow" => ("Hunter Weapons", "Bow"),
        "spear" => ("Hunter Weapons", "Spear"),
        "naturestaff" => ("Hunter Weapons", "Nature Staff"),
        "dagger" => ("Hunter Weapons", "Dagger"),
        "quarterstaff" => ("Hunter Weapons", "Quarterstaff"),
        "torchtype" => ("Hunter Weapons", "Torch"),
        "firestaff" => ("Mage Weapons", "Fire Staff"),
        "holystaff" => ("Mage Weapons", "Holy Staff"),
        "arcanestaff" => ("Mage Weapons", "Arcane Staff"),
        "froststaff" => ("Mage Weapons", "Frost Staff"),
        "cursestaff" => ("Mage Weapons", "Cursed Staff"),
        "booktype" => ("Mage Weapons", "Tome of Spells"),
        "shapeshifterstaff" => ("Hunter Weapons", "Shapeshifter Staff"),
        // Unknown weapon family: no category is invented; it is reported.
        _ => return CategoryPath::new("", None),
    };
    if category.is_empty() {
        return CategoryPath::new("", None);
    }
    CategoryPath::new(category, Some(subcategory))
}

// ---------------------------------------------------------------------------
// Armor: (shop_subcategory1, slot_type) -> (category, subcategory)
// ---------------------------------------------------------------------------

fn armor_path(item: &AodItem) -> CategoryPath {
    let sub1 = item.shop_subcategory1.as_deref().unwrap_or("");
    // Gathering gear (harvester/miner/skinner/fisherman/quarrier/lumberjack)
    // has its own family, split by slot.
    if item.shop_category.as_deref() == Some("gathering") {
        let subcategory = match item.slot_type.as_deref() {
            Some("armor") => "Garb",
            Some("head") => "Cap",
            Some("shoes") => "Workboots",
            _ => return CategoryPath::new("Gathering Armor", None),
        };
        return CategoryPath::new("Gathering Armor", Some(subcategory));
    }
    let (category, subcategory) = match sub1 {
        "cloth_armor" => ("Cloth Armor", "Robes"),
        "cloth_helmet" => ("Cloth Armor", "Cowl"),
        "cloth_shoes" => ("Cloth Armor", "Sandals"),
        "leather_armor" => ("Leather Armor", "Jackets"),
        "leather_helmet" => ("Leather Armor", "Hoods"),
        "leather_shoes" => ("Leather Armor", "Shoes"),
        "plate_armor" => ("Plate Armor", "Armors"),
        "plate_helmet" => ("Plate Armor", "Helmets"),
        "plate_shoes" => ("Plate Armor", "Boots"),
        // Gathering gear and cosmetics have no wiki category yet: nothing is
        // invented, the item is reported as unmapped.
        _ => return CategoryPath::new("", None),
    };
    CategoryPath::new(category, Some(subcategory))
}

// ---------------------------------------------------------------------------
// Accessories: capes, bags, mounts
// ---------------------------------------------------------------------------

fn accessory_path(item: &AodItem) -> CategoryPath {
    match item.slot_type.as_deref() {
        Some("cape") => {
            // Gathering backpacks are stored as capes in the game data.
            if item.shop_category.as_deref() == Some("gathering") {
                return CategoryPath::new("Bag", Some("Backpacks"));
            }
            let sub1 = item.shop_subcategory1.as_deref().unwrap_or("");
            if sub1.contains("avalon") || item.unique_name.contains("_AVALON") {
                CategoryPath::new("Cape", Some("Avalonian Capes"))
            } else if sub1.contains("smuggler") {
                CategoryPath::new("Cape", Some("Smuggler Capes"))
            } else if sub1.contains("brecilien") {
                CategoryPath::new("Cape", Some("Faction Capes"))
            } else if sub1.ends_with("capes_capes") {
                CategoryPath::new("Cape", Some("Normal Capes"))
            } else if item.unique_name.contains("_FW_") || item.unique_name.contains("CAPEITEM_") {
                // Faction capes: the `_FW_` marker (Bridgewatch, Caerleon, …) plus
                // the older faction items (Demon, Heretic, Keeper, Morgana, Undead).
                CategoryPath::new("Cape", Some("Faction Capes"))
            } else {
                // Normal capes keep whatever the dataset already maps them to.
                CategoryPath::new("Cape", None)
            }
        }
        Some("bag") => CategoryPath::new("Bag", Some("Normal Bags")),
        _ => CategoryPath::new("Mount", mount_subcategory(item)),
    }
}

fn mount_subcategory(item: &AodItem) -> Option<&'static str> {
    let sub1 = item.shop_subcategory1.as_deref().unwrap_or("");
    let sub2 = item.shop_subcategory2.as_deref().unwrap_or("");
    let mount_category = item.mount_category.as_deref().unwrap_or("");
    match (sub1, sub2, mount_category) {
        ("basemounts", "horse", _) => Some("Riding Horses"),
        ("basemounts", "ox", _) => Some("Transport Oxen"),
        ("basemounts", "armoredhorse", "ARMORED_HORSES") => Some("Armored Horses"),
        ("basemounts", "armoredhorse", _) => Some("Warhorses"),
        ("basemounts", _, _) => Some("Other Mounts"),
        ("battle_mount", _, _) => Some("Battle Mounts (Season)"),
        ("raremounts", _, _) => Some("Battle Mounts (Other)"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Consumables: foods and potions
// ---------------------------------------------------------------------------

fn consumable_path(item: &AodItem) -> CategoryPath {
    let slot = item.slot_type.as_deref().unwrap_or("");
    let sub2 = item.shop_subcategory2.as_deref().unwrap_or("");
    if slot == "food" {
        let subcategory = match sub2 {
            "omelettes" | "omelette" => Some("Omelette"),
            "pie" | "pies" => Some("Pie"),
            "salad" | "salads" => Some("Salad"),
            "sandwich" | "sandwiches" => Some("Sandwich"),
            "soup" | "soups" => Some("Soup"),
            "stew" | "stews" => Some("Stew"),
            "roast" | "roasts" => Some("Roast"),
            "grilledfish" => Some("Fish"),
            _ => None, // event meals, …
        };
        return CategoryPath::new("Foods", subcategory);
    }
    let subcategory = match sub2 {
        "energy" => Some("Energy"),
        "heal" => Some("Healing"),
        "gigantify" => Some("Gigantify"),
        "resistance" => Some("Resistance"),
        "sticky" | "slowfield" => Some("Sticky"),
        "poison" | "poision" => Some("Poision"),
        "invisible" | "invisibility" => Some("Invisible"),
        "focus" => Some("Focus"),
        "acid" => Some("Acid"),
        "berserk" => Some("Berserk"),
        "cleanse" | "cleansing" => Some("Cleansing"),
        "gather" | "gathering" => Some("Gathering"),
        "lava" | "hellfire" => Some("Hellfire"),
        "lifeward" => Some("Lifeward"),
        "calming" | "mob_reset" => Some("Calming"),
        "tornado" => Some("Tornado"),
        _ => None, // event items
    };
    CategoryPath::new("Potions", subcategory)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, slot: &str, shop: &str, sub1: &str, sub2: &str) -> AodItem {
        AodItem {
            unique_name: id.to_string(),
            name: "X".to_string(),
            description: None,
            tier: Some(4),
            item_power: Some(700),
            slot_type: Some(slot.to_string()),
            shop_category: Some(shop.to_string()),
            shop_subcategory1: Some(sub1.to_string()),
            shop_subcategory2: Some(sub2.to_string()),
            mount_category: None,
            recipes: Vec::new(),
        }
    }

    #[test]
    fn classifies_every_gameplay_slot() {
        assert_eq!(
            table_of(&item("T4_A", "mainhand", "weapons", "axe", "")),
            Some(Table::Weapons)
        );
        assert_eq!(
            table_of(&item("T4_B", "offhand", "offhands", "shieldtype", "")),
            Some(Table::Weapons)
        );
        assert_eq!(
            table_of(&item("T4_C", "armor", "armors", "plate_armor", "")),
            Some(Table::Armors)
        );
        assert_eq!(
            table_of(&item("T4_D", "head", "gathering", "fiber", "")),
            Some(Table::Armors)
        );
        assert_eq!(
            table_of(&item("T4_E", "cape", "capes", "", "")),
            Some(Table::Accessories)
        );
        assert_eq!(
            table_of(&item("T4_F", "bag", "bags", "", "")),
            Some(Table::Accessories)
        );
        assert_eq!(
            table_of(&item("T4_G", "mount", "mounts", "", "")),
            Some(Table::Accessories)
        );
        assert_eq!(
            table_of(&item("T4_H", "food", "consumables", "food", "pie")),
            Some(Table::Consumables)
        );
        assert_eq!(
            table_of(&item("T4_I", "potion", "consumables", "potions", "heal")),
            Some(Table::Consumables)
        );
        assert_eq!(table_of(&item("T4_J", "mount", "furniture", "", "")), None);
    }

    #[test]
    fn maps_weapon_families() {
        let cases = [
            ("sword", "Warrior Weapons", "Sword"),
            ("axe", "Warrior Weapons", "Axe"),
            ("knuckles", "Warrior Weapons", "War Gloves"),
            ("shieldtype", "Warrior Weapons", "Shield"),
            ("bow", "Hunter Weapons", "Bow"),
            ("torchtype", "Hunter Weapons", "Torch"),
            ("booktype", "Mage Weapons", "Tome of Spells"),
            ("shapeshifterstaff", "Hunter Weapons", "Shapeshifter Staff"),
            ("firestaff", "Mage Weapons", "Fire Staff"),
        ];
        for (sub1, category, subcategory) in cases {
            let path = category_path(&item("T4_X", "mainhand", "weapons", sub1, "")).unwrap();
            assert_eq!(path.category, category, "{sub1}");
            assert_eq!(path.subcategory.as_deref(), Some(subcategory), "{sub1}");
        }
    }

    #[test]
    fn maps_armor_slots_and_flags_unknown_sets() {
        let cloth = category_path(&item("T4_X", "armor", "armors", "cloth_armor", "")).unwrap();
        assert_eq!(
            (cloth.category.as_str(), cloth.subcategory.as_deref()),
            ("Cloth Armor", Some("Robes"))
        );

        let gather = category_path(&item(
            "T4_ARMOR_GATHERER_FIBER",
            "armor",
            "gathering",
            "fiber",
            "",
        ))
        .unwrap();
        // Gathering gear has its own family with a slot subcategory.
        assert_eq!(gather.category, "Gathering Armor");
        assert_eq!(gather.subcategory.as_deref(), Some("Garb"));
        let head = category_path(&item("T4_HEAD_GATHERER_ORE", "head", "gathering", "ore", "")).unwrap();
        assert_eq!(head.subcategory.as_deref(), Some("Cap"));
        let shoes = category_path(&item("T4_SHOES_GATHERER_HIDE", "shoes", "gathering", "hide", "")).unwrap();
        assert_eq!(shoes.subcategory.as_deref(), Some("Workboots"));
        // Unknown non-gathering families are still reported, not invented.
        let unknown = category_path(&item("T4_X", "armor", "armors", "mystery", "")).unwrap();
        assert!(unknown.category.is_empty());

        let dragon = category_path(&item(
            "T4_HEAD_LEATHER_DRAGON",
            "head",
            "head",
            "leather_helmet",
            "",
        ))
        .unwrap();
        assert_eq!(dragon.subcategory.as_deref(), Some("Hoods"));
    }

    #[test]
    fn maps_accessories() {
        let faction =
            category_path(&item("T4_CAPEITEM_FW_BRIDGEWATCH", "cape", "capes", "", "")).unwrap();
        assert_eq!(faction.subcategory.as_deref(), Some("Faction Capes"));

        // Non-faction capes are not force-mapped: existing rows keep theirs and
        // new ones are reported as unmapped.
        let normal = category_path(&item("T4_CAPE", "cape", "capes", "", "")).unwrap();
        assert_eq!(normal.category, "Cape");
        assert_eq!(normal.subcategory, None);

        let bag = category_path(&item("T4_BAG", "bag", "bags", "", "")).unwrap();
        assert_eq!(bag.subcategory.as_deref(), Some("Normal Bags"));

        // Gathering backpacks are stored as capes in the game data.
        let backpack = category_path(&item("T4_BACKPACK_GATHERER_FIBER", "cape", "gathering", "fiber", "")).unwrap();
        assert_eq!((backpack.category.as_str(), backpack.subcategory.as_deref()), ("Bag", Some("Backpacks")));

        let avalon = category_path(&item(
            "T4_CAPEITEM_AVALON",
            "cape",
            "capes",
            "accessoires_capes_avalon",
            "",
        ))
        .unwrap();
        assert_eq!(avalon.subcategory.as_deref(), Some("Avalonian Capes"));

        let mut horse = item(
            "T5_MOUNT_ARMORED_HORSE",
            "mount",
            "mounts",
            "basemounts",
            "armoredhorse",
        );
        horse.mount_category = Some("ARMORED_HORSES".to_string());
        assert_eq!(
            category_path(&horse).unwrap().subcategory.as_deref(),
            Some("Armored Horses")
        );

        let mut battle = item("T6_MOUNT_X", "mount", "mounts", "battle_mount", "rhino");
        battle.mount_category = None;
        assert_eq!(
            category_path(&battle).unwrap().subcategory.as_deref(),
            Some("Battle Mounts (Season)")
        );
    }

    #[test]
    fn maps_consumables() {
        let pie =
            category_path(&item("T4_MEAL_PIE", "food", "consumables", "food", "pie")).unwrap();
        assert_eq!(
            (pie.category.as_str(), pie.subcategory.as_deref()),
            ("Foods", Some("Pie"))
        );

        let fish = category_path(&item(
            "T3_MEAL_GRILLEDFISH",
            "food",
            "consumables",
            "food",
            "grilledfish",
        ))
        .unwrap();
        assert_eq!(fish.subcategory.as_deref(), Some("Fish"));
        // Event meals stay unmapped (and are filtered upstream).
        let event = category_path(&item("T4_MEAL_EVENT", "food", "consumables", "food", "event"))
            .unwrap();
        assert_eq!(event.subcategory, None);

        let heal = category_path(&item(
            "T4_POTION_HEAL",
            "potion",
            "consumables",
            "potions",
            "heal",
        ))
        .unwrap();
        assert_eq!(heal.subcategory.as_deref(), Some("Healing"));

        let acid = category_path(&item(
            "T4_POTION_ACID",
            "potion",
            "consumables",
            "potions",
            "acid",
        ))
        .unwrap();
        assert_eq!(acid.subcategory.as_deref(), Some("Acid"));
        let calming =
            category_path(&item("T3_POTION_MOB_RESET", "potion", "consumables", "potions", "calming"))
                .unwrap();
        assert_eq!(calming.subcategory.as_deref(), Some("Calming"));
        let lava = category_path(&item(
            "T4_POTION_LAVA",
            "potion",
            "consumables",
            "potions",
            "lava",
        ))
        .unwrap();
        assert_eq!(lava.subcategory.as_deref(), Some("Hellfire"));
    }
}
