//! DOM parsers: wiki HTML → row drafts.
//!
//! Each parser mirrors one method of `app/Services/DomCrawler/Resources/*`
//! (same CSS selector chains, same fallback order, same text handling).
//! A page that no longer matches its selector chain returns an
//! [`ParseError`]; the command layer turns that into a warning and keeps the
//! existing data.

use scraper::{ElementRef, Html, Selector};
use serde_json::{json, Value};

use crate::fetch::text::*;

/// A page that could not be parsed (site changed, redirect page, …).
#[derive(Debug, Clone)]
pub struct ParseError {
    pub url: String,
    pub reason: String,
}

impl ParseError {
    pub fn new(url: &str, reason: impl Into<String>) -> Self {
        Self {
            url: url.to_string(),
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.url, self.reason)
    }
}

// ---------------------------------------------------------------------------
// Drafts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ListItemDraft {
    pub name: String,
    /// Image URL (only the weapon list exposes one; the base name becomes the
    /// identifier).
    pub icon: Option<String>,
    pub tier: Option<f64>,
    pub item_power: Option<i64>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CategoryDraft {
    pub primary: String,
    pub secondary: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MountDraft {
    pub category: String,
    pub name: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellLinkDraft {
    pub slot: String,
    pub href: String,
}

#[derive(Debug, Clone, Default)]
pub struct SpellDraft {
    pub name: String,
    pub icon: Option<String>,
    pub attributes: Value,
    pub description: Option<String>,
    pub preview: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct StatDraft {
    /// `Item Quality` column when the table has one.
    pub quality: Option<String>,
    /// `Tier` cell text (enchantment is derived from it).
    pub tier: Option<String>,
    /// Remaining columns in table order.
    pub stats: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default)]
pub struct CraftDraft {
    pub per_craft: Option<i64>,
    /// `Tier` cell text.
    pub tier: Option<String>,
    /// Requirement columns in table order (name → amount text).
    pub requirements: Vec<(String, String)>,
}

// ---------------------------------------------------------------------------
// DOM helpers
// ---------------------------------------------------------------------------

fn doc(html: &str) -> Html {
    Html::parse_fragment(html)
}

fn selector(css: &str) -> Selector {
    Selector::parse(css).unwrap_or_else(|e| panic!("invalid selector {css:?}: {e}"))
}

/// Concatenated text content, trimmed (Symfony's `->text()`).
fn text_of(element: &ElementRef) -> String {
    element.text().collect::<String>().trim().to_string()
}

fn attr(element: &ElementRef, name: &str) -> Option<String> {
    element.value().attr(name).map(str::to_string)
}

// ---------------------------------------------------------------------------
// Category list — WeaponResource::categoryList
// ---------------------------------------------------------------------------

pub fn weapon_categories(html: &str, url: &str) -> Result<Vec<CategoryDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let containers = selector("#mw-content-text > div > div > div");
    let h3_span = selector("h3 > span");
    let li_a = selector("ul > li a");

    let mut out = Vec::new();
    for container in root.select(&containers) {
        let primary = container
            .select(&h3_span)
            .next()
            .map(|e| text_of(&e))
            .unwrap_or_default();
        for link in container.select(&li_a) {
            let secondary = text_of(&link);
            if secondary.is_empty() {
                continue;
            }
            out.push(CategoryDraft {
                primary: primary.clone(),
                secondary,
                path: attr(&link, "href"),
            });
        }
    }
    if out.is_empty() {
        return Err(ParseError::new(url, "weapon category list not found"));
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Item lists
// ---------------------------------------------------------------------------

fn draft_row(
    name: String,
    tier: Option<f64>,
    item_power: Option<i64>,
    path: Option<String>,
) -> ListItemDraft {
    ListItemDraft {
        name,
        icon: None,
        tier,
        item_power,
        path,
    }
}

/// `WeaponResource::list` — two possible table layouts.
pub fn weapon_list(html: &str, url: &str) -> Result<Vec<ListItemDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();

    // Layout 1 (all weapons): td0 icon, td1 name+link, td2 tier, td3 item power.
    let layout1 = selector("#mw-content-text > div > div > div:nth-child(1) > table > tbody > tr");
    let rows: Vec<ElementRef> = root.select(&layout1).collect();
    if rows.len() > 1 {
        let td = selector("td");
        let a = selector("a");
        let img = selector("img");
        let mut out = Vec::new();
        for row in rows.iter().skip(1) {
            let cells: Vec<ElementRef> = row.select(&td).collect();
            if cells.len() < 4 {
                continue;
            }
            let name = text_of(&cells[1]);
            if name.is_empty() {
                continue;
            }
            out.push(ListItemDraft {
                name,
                icon: cells[0].select(&img).next().and_then(|i| attr(&i, "src")),
                tier: parse_tier_text(&text_of(&cells[2])),
                item_power: item_power_from_text(&text_of(&cells[3])),
                path: cells[1].select(&a).next().and_then(|a| attr(&a, "href")),
            });
        }
        if !out.is_empty() {
            return Ok(out);
        }
    }

    // Layout 2 (tome of spells / fallback): td0 name (+ optional link/icon).
    let layout2 = selector("#mw-content-text > div > table > tbody > tr");
    let rows: Vec<ElementRef> = root.select(&layout2).collect();
    if rows.len() > 1 {
        let td = selector("td");
        let a = selector("a");
        let img = selector("img");
        let mut out = Vec::new();
        for row in rows.iter().skip(1) {
            let cells: Vec<ElementRef> = row.select(&td).collect();
            if cells.is_empty() {
                continue;
            }
            let name = text_of(&cells[0]);
            if name.is_empty() {
                continue;
            }
            let link = cells[0].select(&a).next().and_then(|a| attr(&a, "href"));
            let path = link.or_else(|| Some(format!("/wiki/{}", name.replace(' ', "_"))));
            out.push(ListItemDraft {
                name: name.clone(),
                icon: cells[0].select(&img).next().and_then(|i| attr(&i, "src")),
                tier: albion_tier(&name),
                item_power: None,
                path,
            });
        }
        if !out.is_empty() {
            return Ok(out);
        }
    }

    Err(ParseError::new(url, "weapon table not found"))
}

/// Shared `td0 name / td1 tier / td2 item power` table parser used by the
/// armor, cape and food lists (each supplies its own selector fallbacks).
pub fn generic_item_list(
    html: &str,
    url: &str,
    selectors: &[&str],
) -> Result<Vec<ListItemDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let td = selector("td");
    let a = selector("a");

    for css in selectors {
        let rows: Vec<ElementRef> = root.select(&selector(css)).collect();
        if rows.len() <= 1 {
            continue;
        }
        let mut out = Vec::new();
        for row in rows.iter().skip(1) {
            let cells: Vec<ElementRef> = row.select(&td).collect();
            if cells.len() < 2 {
                continue;
            }
            let name = text_of(&cells[0]);
            if name.is_empty() {
                continue;
            }
            out.push(ListItemDraft {
                name,
                icon: None,
                tier: parse_tier_text(&text_of(&cells[1])),
                item_power: cells.get(2).and_then(|c| item_power_from_text(&text_of(c))),
                path: cells[0].select(&a).next().and_then(|a| attr(&a, "href")),
            });
        }
        if !out.is_empty() {
            return Ok(out);
        }
    }
    Err(ParseError::new(url, "item table not found"))
}

/// `ArmorResource::list` — three table positions.
pub fn armor_list(html: &str, url: &str) -> Result<Vec<ListItemDraft>, ParseError> {
    generic_item_list(
        html,
        url,
        &[
            "#mw-content-text > div > table:nth-child(5) > tbody > tr",
            "#mw-content-text > div > table:nth-child(13) > tbody > tr",
            "#mw-content-text > div > table > tbody > tr",
        ],
    )
}

/// `AccessoryResource::capeList`.
pub fn cape_list(html: &str, url: &str) -> Result<Vec<ListItemDraft>, ParseError> {
    generic_item_list(
        html,
        url,
        &["#mw-content-text > div > table:nth-child(12) > tbody > tr"],
    )
}

/// `ConsumableResource::foodList`.
pub fn food_list(html: &str, url: &str) -> Result<Vec<ListItemDraft>, ParseError> {
    generic_item_list(html, url, &["#mw-content-text > div > table > tbody > tr"])
}

/// `AccessoryResource::bagList` — links carry the display name in `title`.
pub fn bag_list(html: &str, url: &str) -> Result<Vec<ListItemDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let spans = selector(
        "#mw-content-text > div > table > tbody > tr:nth-child(1) > td:nth-child(2) > span",
    );
    let a = selector("a");

    let spans: Vec<ElementRef> = root.select(&spans).collect();
    if spans.len() <= 1 {
        return Err(ParseError::new(url, "bag table not found"));
    }
    let mut out = Vec::new();
    for span in spans.iter().skip(1) {
        let Some(link) = span.select(&a).next() else {
            continue;
        };
        let name = attr(&link, "title")
            .or_else(|| span.select(&a).next().map(|x| text_of(&x)))
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        out.push(draft_row(
            name.clone(),
            albion_tier(&name),
            None,
            attr(&link, "href"),
        ));
    }
    if out.is_empty() {
        return Err(ParseError::new(url, "bag items not found"));
    }
    Ok(out)
}

/// `AccessoryResource::mountList` — category in the first cell, mounts as
/// spans in the second.
pub fn mount_list(html: &str, url: &str) -> Result<Vec<MountDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let rows = selector("#mw-content-text > div > table:nth-child(11) > tbody > tr");
    let td = selector("td");
    let span = selector("span");
    let a = selector("a");
    let img = selector("img");

    let mut out = Vec::new();
    for row in root.select(&rows) {
        let cells: Vec<ElementRef> = row.select(&td).collect();
        if cells.len() < 2 {
            continue;
        }
        let category = text_of(&cells[0]);
        for item in cells[1].select(&span) {
            let Some(link) = item.select(&a).next() else {
                continue;
            };
            // The original parser used the image file name; the wiki also
            // exposes the human name via `a[title]`, which is what the
            // dataset contains — prefer it and fall back to the image.
            let name = attr(&link, "title")
                .or_else(|| {
                    item.select(&img)
                        .next()
                        .and_then(|i| attr(&i, "src"))
                        .and_then(|src| identifier_from_icon(&src))
                })
                .unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            out.push(MountDraft {
                category: category.clone(),
                name,
                path: attr(&link, "href"),
            });
        }
    }
    if out.is_empty() {
        return Err(ParseError::new(url, "mount table not found"));
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Spells
// ---------------------------------------------------------------------------

/// Weapon slot paragraphs: indexes 1–4 map to Q, W, E, Passive.
pub fn weapon_spell_links(html: &str, url: &str) -> Result<Vec<SpellLinkDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let paragraphs = selector("#mw-content-text > div > p");
    let a = selector("a");
    let paragraphs: Vec<ElementRef> = root.select(&paragraphs).collect();

    let mut out = Vec::new();
    let slots = ["q", "w", "e", "passive"];
    for (index, slot) in slots.iter().enumerate() {
        let Some(paragraph) = paragraphs.get(index + 1) else {
            continue;
        };
        for link in paragraph.select(&a) {
            if let Some(href) = attr(&link, "href") {
                if href.starts_with("/wiki/") {
                    out.push(SpellLinkDraft {
                        slot: slot.to_string(),
                        href,
                    });
                }
            }
        }
    }
    if out.is_empty() {
        return Err(ParseError::new(url, "weapon spell links not found"));
    }
    Ok(out)
}

/// Armor spell paragraphs: indexes 1–2, slot named in the paragraph text.
pub fn armor_spell_links(html: &str, url: &str) -> Result<Vec<SpellLinkDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let paragraphs = selector("#mw-content-text > div > p");
    let a = selector("a");
    let paragraphs: Vec<ElementRef> = root.select(&paragraphs).collect();

    let mut out = Vec::new();
    let mut last_slot: Option<&str> = None;
    for index in 1..=2 {
        let Some(paragraph) = paragraphs.get(index) else {
            continue;
        };
        let text = text_of(paragraph);
        let slot = if text.contains("D spell slot") {
            Some("d")
        } else if text.contains("R spell slot") {
            Some("r")
        } else if text.contains("F spell slot") {
            Some("f")
        } else if text.contains("passive slot") {
            Some("passive")
        } else {
            None
        };
        if slot.is_some() {
            last_slot = slot;
        }
        // NOTE: the Laravel loop keeps `$slot` from the previous iteration
        // when a paragraph has no slot phrase; we mirror that by reusing
        // `last_slot` (and skip if there never was one, to avoid garbage).
        let Some(slot) = last_slot else { continue };
        for link in paragraph.select(&a) {
            if let Some(href) = attr(&link, "href") {
                if href.starts_with("/wiki/") {
                    out.push(SpellLinkDraft {
                        slot: slot.to_string(),
                        href,
                    });
                }
            }
        }
    }
    if out.is_empty() {
        return Err(ParseError::new(url, "armor spell links not found"));
    }
    Ok(out)
}

/// Accessory spell: paragraph 5, when it mentions a unique spell.
pub fn accessory_spell_links(html: &str, _url: &str) -> Result<Vec<SpellLinkDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let paragraph = selector("#mw-content-text > div > p:nth-child(5)");
    let a = selector("a");
    let Some(paragraph) = root.select(&paragraph).next() else {
        return Ok(Vec::new());
    };
    if !text_of(&paragraph).contains("unique spell") {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for link in paragraph.select(&a) {
        if let Some(href) = attr(&link, "href") {
            if href.starts_with("/wiki/") {
                out.push(SpellLinkDraft {
                    slot: "passive".to_string(),
                    href,
                });
            }
        }
    }
    Ok(out)
}

/// Mount spells: list items mentioning "unique abilities".
pub fn mount_spell_links(html: &str, _url: &str) -> Result<Vec<SpellLinkDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let items = selector("#mw-content-text > div > ul > li");
    let a = selector("a");

    let mut out = Vec::new();
    for item in root.select(&items) {
        let text = text_of(&item);
        if !(text.to_lowercase().contains("unique abilites")
            || text.to_lowercase().contains("unique abilities"))
        {
            continue;
        }
        for link in item.select(&a) {
            if let Some(href) = attr(&link, "href") {
                if href.contains("/wiki/") {
                    out.push(SpellLinkDraft {
                        slot: "mount".to_string(),
                        href,
                    });
                }
            }
        }
    }
    Ok(out)
}

/// `SpellResource::detail` (+ the shared spell blocks of the weapon/armor/
/// accessory resources).
pub fn spell_detail(html: &str, url: &str) -> Result<SpellDraft, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let table_4 = selector("#mw-content-text > div > table:nth-child(4) > tbody > tr");
    let table_5 = selector("#mw-content-text > div > table:nth-child(5) > tbody > tr");
    let rows: Vec<ElementRef> = if root.select(&table_4).count() > 0 {
        root.select(&table_4).collect()
    } else {
        root.select(&table_5).collect()
    };
    if rows.is_empty() {
        return Err(ParseError::new(url, "spell attribute table not found"));
    }
    let td = selector("td");

    let mut attributes = Vec::new();
    let limit = rows.len().saturating_sub(1); // range(0, count-2) inclusive
    for (index, row) in rows.iter().take(limit).enumerate() {
        let cells: Vec<ElementRef> = row.select(&td).collect();
        let (name_index, value_index) = if index > 0 { (0, 1) } else { (1, 2) };
        if cells.get(name_index).is_none() {
            continue;
        }
        attributes.push(json!({
            "name": text_of(&cells[name_index]),
            "value": cells.get(value_index).map(text_of).unwrap_or_default(),
        }));
    }

    let name_link = selector("td:nth-child(1) > span > a");
    let name = rows[0]
        .select(&name_link)
        .next()
        .map(|e| text_of(&e))
        .unwrap_or_default();
    let icon = rows[0]
        .select(&name_link)
        .next()
        .and_then(|a| a.select(&selector("img")).next())
        .and_then(|i| attr(&i, "src"));

    let description = rows[0]
        .select(&selector("td:nth-child(4)"))
        .next()
        .map(|e| e.inner_html())
        .filter(|html| !html.trim().is_empty());

    let preview = root
        .select(&selector(
            "#mw-content-text > div > p:nth-child(9) > a > img",
        ))
        .next()
        .and_then(|img| attr(&img, "src"))
        .map(|src| wiki_preview(&src));

    if name.is_empty() {
        return Err(ParseError::new(url, "spell name not found"));
    }

    Ok(SpellDraft {
        name,
        icon,
        attributes: Value::Array(attributes),
        description,
        preview,
    })
}

// ---------------------------------------------------------------------------
// Stat tables
// ---------------------------------------------------------------------------

/// Stat-table selector fallbacks shared by weapons, armors and accessories.
const STAT_TABLE_SELECTORS: &[&str] = &[
    "#mw-content-text > div > table:nth-child(14)",
    "#mw-content-text > div > table:nth-child(18)",
    "#mw-content-text > div > table:nth-child(15)",
    "#mw-content-text > div > table:nth-child(10)",
    "#mw-content-text > div > table.wikitable.sortable.jquery-tablesorter",
    "#mw-content-text > div > table.wikitable.sortable",
];

fn find_stat_table<'a>(root: &'a ElementRef<'a>, marker: &str) -> Option<ElementRef<'a>> {
    for css in STAT_TABLE_SELECTORS {
        if let Some(table) = root.select(&selector(css)).next() {
            if text_of(&table).contains(marker) {
                return Some(table);
            }
        }
    }
    None
}

fn stat_rows_from_table(table: &ElementRef, force_normal_quality: bool) -> Vec<StatDraft> {
    let th = selector("th");
    let td = selector("td");
    let tr = selector("tr");

    // Laravel collects every <th> in the table and drops the first one; data
    // rows are indexed against that header list 1:1 (the icon column exists
    // only in the header).
    let header_cells: Vec<ElementRef> = table.select(&th).collect();
    let mut headers: Vec<String> = Vec::new();
    if force_normal_quality {
        headers.push("Item Quality".to_string());
    }
    for cell in header_cells.iter().skip(1) {
        headers.push(text_of(cell));
    }

    let rows: Vec<ElementRef> = table.select(&tr).collect();
    let mut out = Vec::new();
    for row in rows.iter().skip(2) {
        let cells: Vec<ElementRef> = row.select(&td).collect();
        let mut quality: Option<String> = if force_normal_quality {
            Some("Normal".to_string())
        } else {
            None
        };
        let mut tier: Option<String> = None;
        let mut stats = Vec::new();
        for (index, cell) in cells.iter().enumerate() {
            // The accessory tables squeeze in a synthetic quality column, so
            // their data cells are offset by one header.
            let header_index = if force_normal_quality {
                index + 1
            } else {
                index
            };
            let Some(header) = headers.get(header_index) else {
                continue;
            };
            let value = text_of(cell);
            match header.as_str() {
                "Item Quality" => quality = Some(value),
                "Tier" => tier = Some(value),
                other => stats.push((other.to_string(), value)),
            }
        }
        if quality.as_deref().is_some_and(|q| q.contains("Version")) {
            continue;
        }
        if quality.is_none() && tier.is_none() {
            continue;
        }
        out.push(StatDraft {
            quality,
            tier,
            stats,
        });
    }
    out
}

/// `WeaponResource::statList` / `ArmorResource::statList`.
pub fn weapon_or_armor_stats(html: &str, url: &str) -> Result<Vec<StatDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let table = find_stat_table(&root, "Item Quality")
        .ok_or_else(|| ParseError::new(url, "stat table not found"))?;
    Ok(stat_rows_from_table(&table, false))
}

/// `AccessoryResource::statList` — mount tables may lack the quality column.
pub fn accessory_stats(html: &str, url: &str) -> Result<Vec<StatDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let table = find_stat_table(&root, "Mount Hit Points")
        .ok_or_else(|| ParseError::new(url, "mount stat table not found"))?;
    let th = selector("th");
    let header_cells: Vec<ElementRef> = table.select(&th).collect();
    let force = header_cells
        .get(1)
        .map(|e| text_of(e) != "Item Quality")
        .unwrap_or(true);
    Ok(stat_rows_from_table(&table, force))
}

/// Transposed consumable stat table (`foodStatList` / `potionStatList`):
/// columns are tiers, rows are stat names.
fn transposed_stats(
    html: &str,
    url: &str,
    css: &str,
    label: &str,
) -> Result<Vec<StatDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let tr = selector("tr");
    let th = selector("th");
    let td = selector("td");
    let table = root
        .select(&selector(css))
        .next()
        .ok_or_else(|| ParseError::new(url, format!("{label} table not found")))?;
    let rows: Vec<ElementRef> = table.select(&tr).collect();
    if rows.len() < 3 {
        return Err(ParseError::new(
            url,
            format!("{label} table has too few rows"),
        ));
    }
    let header_cells: Vec<ElementRef> = rows[1].select(&th).collect();
    let col_count = header_cells.len();
    let mut out = Vec::new();
    for col in 0..col_count {
        let tier = header_cells.get(col).map(text_of).unwrap_or_default();
        let mut draft = StatDraft {
            quality: Some("Normal".to_string()),
            tier: Some(tier.replace("Tier ", "")),
            stats: Vec::new(),
        };
        for row in rows.iter().skip(2) {
            let cells: Vec<ElementRef> = row.select(&td).collect();
            let header = cells.first().map(text_of).unwrap_or_default();
            let value = cells.get(col + 1).map(text_of).unwrap_or_default();
            draft.stats.push((header, value));
        }
        out.push(draft);
    }
    Ok(out)
}

/// `ConsumableResource::foodStatList`.
pub fn food_stats(html: &str, url: &str) -> Result<Vec<StatDraft>, ParseError> {
    transposed_stats(
        html,
        url,
        "#mw-content-text > div > table:nth-child(12)",
        "food stat",
    )
}

/// `ConsumableResource::potionStatList`.
pub fn potion_stats(html: &str, url: &str) -> Result<Vec<StatDraft>, ParseError> {
    transposed_stats(
        html,
        url,
        "#mw-content-text > div > table:nth-child(10)",
        "potion stat",
    )
}

// ---------------------------------------------------------------------------
// Consumable crafting & info
// ---------------------------------------------------------------------------

/// `…per craft: N (` from the page paragraphs.
fn per_craft_from_page(root: &ElementRef) -> Option<i64> {
    let paragraphs = selector("#mw-content-text > div > p");
    let paragraphs: Vec<ElementRef> = root.select(&paragraphs).collect();
    let paragraph = paragraphs.get(paragraphs.len().checked_sub(2)?)?;
    let text = text_of(paragraph);
    let after = text.split("per craft: ").nth(1)?;
    let value = after.split(" (").next().unwrap_or(after).trim();
    value.parse::<i64>().ok()
}

fn transposed_crafting(
    html: &str,
    url: &str,
    css: &str,
    label: &str,
) -> Result<Vec<CraftDraft>, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let tr = selector("tr");
    let th = selector("th");
    let td = selector("td");
    let per_craft = per_craft_from_page(&root);
    let table = root
        .select(&selector(css))
        .next()
        .ok_or_else(|| ParseError::new(url, format!("{label} table not found")))?;
    let rows: Vec<ElementRef> = table.select(&tr).collect();
    if rows.len() < 3 {
        return Err(ParseError::new(
            url,
            format!("{label} table has too few rows"),
        ));
    }
    let header_cells: Vec<ElementRef> = rows[1].select(&th).collect();
    let col_count = header_cells.len();
    let mut out = Vec::new();
    for col in 0..col_count {
        let tier = header_cells.get(col).map(text_of).unwrap_or_default();
        let mut requirements = Vec::new();
        for row in rows.iter().skip(2) {
            let cells: Vec<ElementRef> = row.select(&td).collect();
            let header = cells.first().map(text_of).unwrap_or_default();
            let value = cells.get(col + 1).map(text_of).unwrap_or_default();
            if !header.is_empty() {
                requirements.push((header, value));
            }
        }
        out.push(CraftDraft {
            per_craft,
            tier: Some(tier.replace("Tier ", "")),
            requirements,
        });
    }
    Ok(out)
}

/// `ConsumableResource::foodCraftingList`.
pub fn food_crafting(html: &str, url: &str) -> Result<Vec<CraftDraft>, ParseError> {
    transposed_crafting(
        html,
        url,
        "#mw-content-text > div > table:nth-child(10)",
        "food crafting",
    )
}

/// `ConsumableResource::potionCraftingList`.
pub fn potion_crafting(html: &str, url: &str) -> Result<Vec<CraftDraft>, ParseError> {
    transposed_crafting(
        html,
        url,
        "#mw-content-text > div > table:nth-child(8)",
        "potion crafting",
    )
}

/// `ConsumableResource::info`: paragraphs 1…n-2, one per line.
pub fn consumable_info(html: &str, url: &str) -> Result<String, ParseError> {
    let document = doc(html);
    let root = document.root_element();
    let paragraphs = selector("#mw-content-text > div > p");
    let paragraphs: Vec<ElementRef> = root.select(&paragraphs).collect();
    if paragraphs.len() < 3 {
        return Err(ParseError::new(url, "info paragraphs not found"));
    }
    let mut info = String::new();
    for paragraph in paragraphs
        .iter()
        .skip(1)
        .take(paragraphs.len().saturating_sub(2))
    {
        info.push_str(&text_of(paragraph));
        info.push('\n');
    }
    Ok(info)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_weapon_category_tree() {
        let html = r#"
        <div id="mw-content-text"><div><div><div>
            <h3><span>Warrior Weapons</span></h3>
            <ul><li><a href="/wiki/Axe">Axe</a></li><li><a href="/wiki/Sword">Sword</a></li></ul>
        </div></div></div></div>"#;
        let cats = weapon_categories(html, "test").unwrap();
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].primary, "Warrior Weapons");
        assert_eq!(cats[0].secondary, "Axe");
        assert_eq!(cats[0].path.as_deref(), Some("/wiki/Axe"));
    }

    #[test]
    fn parses_weapon_list_layout_one() {
        let html = r#"
        <div id="mw-content-text"><div><div><div>
            <table><tbody>
                <tr><th>Icon</th><th>Name</th><th>Tier</th><th>Item Power</th></tr>
                <tr>
                    <td><img src="https://cdn.example/T4_MAIN_AXE.png"></td>
                    <td><a href="/wiki/Adept%27s_Battleaxe">Adept's Battleaxe</a></td>
                    <td>4</td><td>700</td>
                </tr>
            </tbody></table>
        </div></div></div></div>"#;
        let items = weapon_list(html, "test").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Adept's Battleaxe");
        assert_eq!(
            items[0].icon.as_deref(),
            Some("https://cdn.example/T4_MAIN_AXE.png")
        );
        assert_eq!(items[0].tier, Some(4.0));
        assert_eq!(items[0].item_power, Some(700));
        assert_eq!(items[0].path.as_deref(), Some("/wiki/Adept%27s_Battleaxe"));
    }

    #[test]
    fn parses_weapon_list_layout_two() {
        let html = r#"
        <div id="mw-content-text"><div>
            <table><tbody>
                <tr><th>Spell</th></tr>
                <tr><td><a href="/wiki/Rending_Strike">Rending Strike</a></td></tr>
            </tbody></table>
        </div></div>"#;
        let items = weapon_list(html, "test").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Rending Strike");
        assert_eq!(items[0].tier, None);
    }

    #[test]
    fn parses_armor_list_fallback_positions() {
        let html = r#"
        <div id="mw-content-text"><div>
            <table><tbody>
                <tr><th>Name</th><th>Tier</th><th>Item Power</th></tr>
                <tr><td><a href="/wiki/Adept%27s_Plate_Armor">Adept's Plate Armor</a></td><td>4</td><td>700</td></tr>
            </tbody></table>
        </div></div>"#;
        let items = armor_list(html, "test").unwrap();
        assert_eq!(items[0].name, "Adept's Plate Armor");
        assert_eq!(items[0].tier, Some(4.0));
        assert_eq!(items[0].item_power, Some(700));
    }

    #[test]
    fn parses_bag_list_from_span_links() {
        let html = r#"
        <div id="mw-content-text"><div>
            <table><tbody><tr>
                <td>Bags</td>
                <td>
                    <span>header</span>
                    <span><a href="/wiki/Expert%27s_Bag" title="Expert's Bag"><img src="x.png"></a></span>
                </td>
            </tr></tbody></table>
        </div></div>"#;
        let items = bag_list(html, "test").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Expert's Bag");
        assert_eq!(items[0].tier, Some(5.0));
    }

    #[test]
    fn parses_mount_list_preferring_link_title() {
        // The real page has table:nth-child(11); build it exactly.
        let html = format!("<div id=\"mw-content-text\"><div>{}</div></div>", {
            let mut tables = String::new();
            for _ in 0..10 {
                tables.push_str("<table><tbody><tr><td>x</td></tr></tbody></table>");
            }
            tables.push_str(
                    r#"<table><tbody>
                        <tr><td>Armored Horses</td><td>
                            <span><a href="/wiki/Expert%27s_Armored_Horse" title="Expert's Armored Horse"><img src="T5_MOUNT_ARMORED_HORSE.png"></a></span>
                        </td></tr>
                    </tbody></table>"#,
                );
            tables
        });
        let _ = html; // silence the unused first fixture
        let mounts = mount_list(&html, "test").unwrap();
        assert_eq!(mounts.len(), 1);
        assert_eq!(mounts[0].category, "Armored Horses");
        assert_eq!(mounts[0].name, "Expert's Armored Horse");
    }

    #[test]
    fn parses_weapon_spell_links_in_slot_order() {
        let html = r#"
        <div id="mw-content-text"><div>
            <p>intro</p>
            <p><a href="/wiki/Rending_Strike">Rending Strike</a></p>
            <p><a href="/wiki/Cleave">Cleave</a></p>
            <p><a href="/wiki/Whirlwind">Whirlwind</a></p>
            <p><a href="/wiki/Axe_Passive">Axe Passive</a></p>
        </div></div>"#;
        let links = weapon_spell_links(html, "test").unwrap();
        assert_eq!(links.len(), 4);
        assert_eq!(links[0].slot, "q");
        assert_eq!(links[1].slot, "w");
        assert_eq!(links[2].slot, "e");
        assert_eq!(links[3].slot, "passive");
        assert_eq!(links[0].href, "/wiki/Rending_Strike");
    }

    #[test]
    fn parses_spell_detail() {
        let html = r#"
        <div id="mw-content-text"><div>
            <table><tbody><tr><td>filler 1</td></tr></tbody></table>
            <table><tbody><tr><td>filler 2</td></tr></tbody></table>
            <table><tbody><tr><td>filler 3</td></tr></tbody></table>
            <table><tbody>
                <tr>
                    <td><span><a href="/wiki/Slash"><img src="/images/slash.png">Slash</a></span></td>
                    <td>Slash</td><td>Energy Cost</td><td>Deals <b>damage</b></td>
                </tr>
                <tr><td>Energy Cost</td><td>6</td></tr>
                <tr><td>Cooldown</td><td>2s</td></tr>
            </tbody></table>
            <p>one</p><p>two</p><p>three</p><p>four</p>
            <p><a href="/images/thumb/preview.png"><img src="/images/thumb/preview.png"></a></p>
        </div></div>"#;
        let detail = spell_detail(html, "test").unwrap();
        assert_eq!(detail.name, "Slash");
        assert_eq!(detail.icon.as_deref(), Some("/images/slash.png"));
        // The last attribute row is skipped, exactly like the Laravel loop
        // (`range(0, count - 2)`).
        assert_eq!(
            detail.attributes,
            serde_json::json!([
                {"name": "Slash", "value": "Energy Cost"},
                {"name": "Energy Cost", "value": "6"}
            ])
        );
        assert_eq!(
            detail.preview.as_deref(),
            Some("https://wiki.albiononline.com/images/thumb/preview.png")
        );
        assert_eq!(detail.description.as_deref(), Some("Deals <b>damage</b>"));
    }

    #[test]
    fn parses_weapon_stats_at_fallback_table_position() {
        let mut tables = String::new();
        for _ in 0..13 {
            tables.push_str("<table><tbody><tr><td>x</td></tr></tbody></table>");
        }
        // Header row keeps an icon column (first <th>, skipped); data rows do
        // not have it — exactly the layout the Laravel index mapping assumes.
        tables.push_str(
            r#"<table><tbody>
                <tr><th>Icon</th><th>Item Quality</th><th>Tier</th><th>Physical damage</th><th>Max Hit Points</th></tr>
                <tr><td>Version 4.0</td><td>4.0</td><td>0</td><td>0</td></tr>
                <tr><td>Normal</td><td>4.0</td><td>61</td><td>67</td></tr>
                <tr><td>Good</td><td>4.1</td><td>72</td><td>71</td></tr>
                <tr><td>Version 4.1</td><td>4.1</td><td>0</td><td>0</td></tr>
            </tbody></table>"#,
        );
        let html = format!("<div id=\"mw-content-text\"><div>{tables}</div></div>");
        let stats = weapon_or_armor_stats(&html, "test").unwrap();
        // Row 1 (a repeated header/version row) is skipped by the index
        // offset; the trailing "Version 4.1" data row is skipped by text.
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].quality.as_deref(), Some("Normal"));
        assert_eq!(stats[0].tier.as_deref(), Some("4.0"));
        assert_eq!(
            stats[0].stats,
            vec![
                ("Physical damage".to_string(), "61".to_string()),
                ("Max Hit Points".to_string(), "67".to_string())
            ]
        );
        assert_eq!(stats[1].quality.as_deref(), Some("Good"));
        assert_eq!(stats[1].tier.as_deref(), Some("4.1"));
    }

    #[test]
    fn parses_transposed_food_stats() {
        let mut tables = String::new();
        for _ in 0..11 {
            tables.push_str("<table><tbody><tr><td>x</td></tr></tbody></table>");
        }
        tables.push_str(
            r#"<table><tbody>
                <tr><th>x</th><th>Tier 4.0</th><th>Tier 4.1</th></tr>
                <tr><th>Tier 4.0</th><th>Tier 4.1</th></tr>
                <tr><td>Item Power</td><td>500</td><td>700</td></tr>
                <tr><td>Nutrition</td><td>87</td><td>87</td></tr>
            </tbody></table>"#,
        );
        let html = format!("<div id=\"mw-content-text\"><div>{tables}</div></div>");
        let stats = food_stats(&html, "test").unwrap();
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].tier.as_deref(), Some("4.0"));
        assert_eq!(stats[1].tier.as_deref(), Some("4.1"));
        assert_eq!(
            stats[0].stats,
            vec![
                ("Item Power".to_string(), "500".to_string()),
                ("Nutrition".to_string(), "87".to_string())
            ]
        );
        assert_eq!(
            stats[1].stats[0],
            ("Item Power".to_string(), "700".to_string())
        );
    }

    #[test]
    fn parses_consumable_info() {
        let html = r#"
        <div id="mw-content-text"><div>
            <p>title</p>
            <p>first paragraph</p>
            <p>second paragraph</p>
            <p>per craft: 10 (Station)</p>
        </div></div>"#;
        let info = consumable_info(html, "test").unwrap();
        assert_eq!(info, "first paragraph\nsecond paragraph\n");
    }

    #[test]
    fn parses_crafting_with_per_craft() {
        let mut tables = String::new();
        for _ in 0..4 {
            tables.push_str("<table><tbody><tr><td>x</td></tr></tbody></table>");
        }
        tables.push_str(
            r#"<table><tbody>
                <tr><th>x</th><th>Tier 4.0</th></tr>
                <tr><th>Tier 4.0</th></tr>
                <tr><td>Raw Chicken</td><td>8</td></tr>
                <tr><td>Goat's Milk</td><td>4</td></tr>
            </tbody></table>"#,
        );
        // `per craft: N (` is read from the second-to-last paragraph; the
        // table is the 8th element child (3 paragraphs + 4 filler tables).
        let html = format!(
            "<div id=\"mw-content-text\"><div><p>intro</p><p>per craft: 10 (Station)</p><p>footer</p>{tables}</div></div>"
        );
        let crafts = transposed_crafting(
            &html,
            "test",
            "#mw-content-text > div > table:nth-child(8)",
            "potion crafting",
        )
        .expect("crafting parses");
        assert_eq!(crafts.len(), 1);
        assert_eq!(crafts[0].per_craft, Some(10));
        assert_eq!(crafts[0].tier.as_deref(), Some("4.0"));
        assert_eq!(
            crafts[0].requirements,
            vec![
                ("Raw Chicken".to_string(), "8".to_string()),
                ("Goat's Milk".to_string(), "4".to_string())
            ]
        );
    }
}
