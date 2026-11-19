//! Hard-coded seeds from the Laravel commands.
//!
//! The wiki has no machine-readable category index for armor/accessories/
//! consumables and potions, so the original commands embed fixed lists. They
//! are reproduced here verbatim (including the `Poision` / `Gigantify`
//! spellings, which the dataset references by name).

/// `GetArmorCategory`: (primary, secondary, secondary wiki path).
pub const ARMOR_CATEGORIES: &[(&str, &str, &str)] = &[
    ("Cloth Armor", "Robes", "/wiki/Cloth_Robes"),
    ("Cloth Armor", "Cowl", "/wiki/Cloth_Cowls"),
    ("Cloth Armor", "Sandals", "/wiki/Cloth_Sandals"),
    ("Leather Armor", "Jackets", "/wiki/Leather_Jackets"),
    ("Leather Armor", "Hoods", "/wiki/Leather_Hoods"),
    ("Leather Armor", "Shoes", "/wiki/Leather_Shoes"),
    ("Plate Armor", "Armors", "/wiki/Plate_Armors"),
    ("Plate Armor", "Helmets", "/wiki/Plate_Helmets"),
    ("Plate Armor", "Boots", "/wiki/Plate_Boots"),
];

/// `GetAccessoryCategory`: (primary, secondary, secondary wiki path).
pub const ACCESSORY_CATEGORIES: &[(&str, &str, &str)] = &[
    ("Cape", "Normal Capes", "/wiki/Cape"),
    ("Cape", "Faction Capes", "/wiki/Cape"),
    ("Bag", "Normal Bags", "/wiki/Bag"),
];

/// `GetConsumableCategory`: (primary, secondary, path) — all `version = 2`.
pub const CONSUMABLE_CATEGORIES: &[(&str, &str, Option<&str>)] = &[
    ("Foods", "Omelette", Some("/wiki/Omelette")),
    ("Foods", "Pie", Some("/wiki/Pie")),
    ("Foods", "Salad", Some("/wiki/Salad")),
    ("Foods", "Sandwich", Some("/wiki/Sandwich")),
    ("Foods", "Soup", Some("/wiki/Soup")),
    ("Foods", "Stew", Some("/wiki/Stew")),
    ("Foods", "Roast", Some("/wiki/Roast")),
    ("Potions", "Energy", None),
    ("Potions", "Healing", None),
    ("Potions", "Gigantify", None),
    ("Potions", "Resistance", None),
    ("Potions", "Sticky", None),
    ("Potions", "Poision", None),
    ("Potions", "Invisible", None),
    ("Potions", "Focus", None),
];

/// `GetPotion`: (name, tier, item power, wiki path, subcategory name).
pub const POTIONS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "Minor Energy Potion",
        "2",
        "300",
        "/wiki/Minor_Energy_Potion",
        "Energy",
    ),
    (
        "Minor Healing Potion",
        "2",
        "300",
        "/wiki/Minor_Healing_Potion",
        "Healing",
    ),
    (
        "Minor Gigantify Potion",
        "3",
        "500",
        "/wiki/Minor_Gigantify_Potion",
        "Gigantify",
    ),
    (
        "Minor Resistance Potion",
        "3",
        "500",
        "/wiki/Minor_Resistance_Potion",
        "Resistance",
    ),
    (
        "Minor Sticky Potion",
        "3",
        "500",
        "/wiki/Minor_Sticky_Potion",
        "Sticky",
    ),
    ("Energy Potion", "4", "700", "/wiki/Energy_Potion", "Energy"),
    (
        "Healing Potion",
        "4",
        "700",
        "/wiki/Healing_Potion",
        "Healing",
    ),
    (
        "Minor Poison Potion",
        "4",
        "700",
        "/wiki/Minor_Poison_Potion",
        "Poision",
    ),
    (
        "Gigantify Potion",
        "5.0",
        "800",
        "/wiki/Gigantify_Potion",
        "Gigantify",
    ),
    (
        "Resistance Potion",
        "5.0",
        "800",
        "/wiki/Resistance_Potion",
        "Resistance",
    ),
    (
        "Sticky Potion",
        "5.0",
        "800",
        "/wiki/Sticky_Potion",
        "Sticky",
    ),
    (
        "Major Energy Potion",
        "6.0",
        "900",
        "/wiki/Major_Energy_Potion",
        "Energy",
    ),
    (
        "Major Healing Potion",
        "6.0",
        "900",
        "/wiki/Major_Healing_Potion",
        "Healing",
    ),
    (
        "Poison Potion",
        "6.0",
        "900",
        "/wiki/Poison_Potion",
        "Poision",
    ),
    (
        "Major Gigantify Potion",
        "7.0",
        "980",
        "/wiki/Major_Gigantify_Potion",
        "Gigantify",
    ),
    (
        "Major Resistance Potion",
        "7.0",
        "980",
        "/wiki/Major_Resistance_Potion",
        "Resistance",
    ),
    (
        "Major Sticky Potion",
        "7.0",
        "980",
        "/wiki/Major_Sticky_Potion",
        "Sticky",
    ),
    (
        "Focus Restoration Potion",
        "8.0",
        "0",
        "/wiki/Focus_Restoration_Potion",
        "Focus",
    ),
    (
        "Invisibility Potion",
        "8.0",
        "1060",
        "/wiki/Invisibility_Potion",
        "Invisible",
    ),
    (
        "Major Poison Potion",
        "8.0",
        "1060",
        "/wiki/Major_Poison_Potion",
        "Poision",
    ),
];

/// `GetMount`: the primary mount category and its wiki path.
pub const MOUNT_CATEGORY: (&str, &str) = ("Mount", "/wiki/Cloth_Robes");

/// The page whose mount table `GetMount` crawls.
pub const MOUNT_PAGE: &str = "/wiki/Adept%27s_Giant_Stag";

pub const CAPE_PAGE: &str = "/wiki/Cape";
pub const BAG_PAGE: &str = "/wiki/Bag";

/// `GetWeaponCategory` crawls this page for the weapon category tree.
pub const WEAPON_CATEGORY_PAGE: &str = "/wiki/Weapon";

/// `GetWeaponStat` skips this known-broken item page.
pub const WEAPON_STAT_SKIP_PATHS: &[&str] = &["/wiki/Journeyman's_Nature_Staff"];

/// `GetAccessoryStat` skips these known-broken mount pages.
pub const ACCESSORY_STAT_SKIP_PATHS: &[&str] = &["/wiki/Yule_Ram", "/wiki/Bronze_Battle_Rhino"];

/// `GetConsumableStat` / `GetConsumableCrafting` special-case this potion: it
/// has no stats table on the wiki, so the stat row is seeded directly.
pub const FOCUS_RESTORATION_POTION: &str = "Focus Restoration Potion";

/// The seeded stat row for [`FOCUS_RESTORATION_POTION`].
pub fn focus_restoration_stats() -> Vec<(String, String)> {
    vec![
        ("Item Power".to_string(), "0".to_string()),
        ("Weight".to_string(), "0.1 kg".to_string()),
        ("Restore Focus".to_string(), "+10000".to_string()),
    ]
}
