//! In-repo documentation: the page registry, sidebar, table of contents and
//! page rendering. Content lives as Markdown in `rust/web/docs/` and is
//! embedded in the binary (and the Workers wasm build).

use std::sync::OnceLock;

use crate::web::layout::{self, Active, PageMeta, SITE_URL};
use crate::web::markdown::{self, Heading};

#[derive(Debug)]
pub struct Doc {
    pub slug: &'static str,
    pub title: &'static str,
    pub group: &'static str,
    pub summary: &'static str,
    pub method: Option<&'static str>,
    pub endpoint: Option<&'static str>,
    pub demo: Option<&'static str>,
    pub body: &'static str,
}

pub const DOCS: &[Doc] = &[
    Doc {
        slug: "introduction",
        title: "Introduction",
        group: "Getting started",
        summary: "The base URL, response envelope and a first request.",
        method: None,
        endpoint: None,
        demo: None,
        body: include_str!("../../web/docs/introduction.md"),
    },
    Doc {
        slug: "versions",
        title: "Versions & extras",
        group: "Getting started",
        summary: "API versions, rate limits, errors and the Weaponry endpoints.",
        method: None,
        endpoint: None,
        demo: None,
        body: include_str!("../../web/docs/versions.md"),
    },
    Doc {
        slug: "categories",
        title: "Categories",
        group: "Endpoints",
        summary: "Category and subcategory trees for every item type.",
        method: Some("GET"),
        endpoint: Some("/api/v3/categories"),
        demo: Some("/api/v3/categories?type=weapon"),
        body: include_str!("../../web/docs/categories.md"),
    },
    Doc {
        slug: "weapons",
        title: "Weapons",
        group: "Endpoints",
        summary: "List weapons and filter by category, subcategory or tier.",
        method: Some("GET"),
        endpoint: Some("/api/v3/weapons"),
        demo: Some("/api/v3/weapons?tier=8"),
        body: include_str!("../../web/docs/weapons.md"),
    },
    Doc {
        slug: "weapon-stats",
        title: "Weapon Stats",
        group: "Endpoints",
        summary: "Per-quality weapon stats grouped by enchantment.",
        method: Some("GET"),
        endpoint: Some("/api/v3/weapon-stats/weapon/{weapon_id}"),
        demo: Some("/api/v3/weapon-stats/weapon/1"),
        body: include_str!("../../web/docs/weapon-stats.md"),
    },
    Doc {
        slug: "armors",
        title: "Armors",
        group: "Endpoints",
        summary: "List armors and filter by category, subcategory or tier.",
        method: Some("GET"),
        endpoint: Some("/api/v3/armors"),
        demo: Some("/api/v3/armors?tier=4"),
        body: include_str!("../../web/docs/armors.md"),
    },
    Doc {
        slug: "armor-stats",
        title: "Armor Stats",
        group: "Endpoints",
        summary: "Per-quality armor stats grouped by enchantment.",
        method: Some("GET"),
        endpoint: Some("/api/v3/armor-stats/armor/{armor_id}"),
        demo: Some("/api/v3/armor-stats/armor/1"),
        body: include_str!("../../web/docs/armor-stats.md"),
    },
    Doc {
        slug: "accessories",
        title: "Accessories",
        group: "Endpoints",
        summary: "Capes, bags and mounts, with the same filters as items.",
        method: Some("GET"),
        endpoint: Some("/api/v3/accessories"),
        demo: Some("/api/v3/accessories?tier=4"),
        body: include_str!("../../web/docs/accessories.md"),
    },
    Doc {
        slug: "accessories-stats",
        title: "Accessory Stats",
        group: "Endpoints",
        summary: "Per-quality accessory stats grouped by enchantment.",
        method: Some("GET"),
        endpoint: Some("/api/v3/accessory-stats/accessory/{accessory_id}"),
        demo: Some("/api/v3/accessory-stats/accessory/1"),
        body: include_str!("../../web/docs/accessories-stats.md"),
    },
    Doc {
        slug: "consumables",
        title: "Consumables",
        group: "Endpoints",
        summary: "Food and potions with tier, item power and item info.",
        method: Some("GET"),
        endpoint: Some("/api/v3/consumables"),
        demo: Some("/api/v3/consumables?tier=4"),
        body: include_str!("../../web/docs/consumables.md"),
    },
    Doc {
        slug: "consumable-stats",
        title: "Consumable Stats",
        group: "Endpoints",
        summary: "Per-quality consumable stats grouped by enchantment.",
        method: Some("GET"),
        endpoint: Some("/api/v3/consumable-stats/consumable/{consumable_id}"),
        demo: Some("/api/v3/consumable-stats/consumable/1"),
        body: include_str!("../../web/docs/consumable-stats.md"),
    },
    Doc {
        slug: "consumable-craftings",
        title: "Consumable Craftings",
        group: "Endpoints",
        summary: "Crafting recipes, ingredients and per-craft output.",
        method: Some("GET"),
        endpoint: Some("/api/v3/consumable-craftings/consumable/{consumable_id}"),
        demo: Some("/api/v3/consumable-craftings/consumable/1"),
        body: include_str!("../../web/docs/consumable-craftings.md"),
    },
    Doc {
        slug: "spells",
        title: "Spells",
        group: "Endpoints",
        summary: "Spells for weapons, armors and accessories, grouped by slot.",
        method: Some("GET"),
        endpoint: Some("/api/v3/spells/weapon/{weapon_id}"),
        demo: Some("/api/v3/spells/weapon/1"),
        body: include_str!("../../web/docs/spells.md"),
    },
    Doc {
        slug: "category-types",
        title: "Category Types",
        group: "Types",
        summary: "The four item types used by the categories endpoint.",
        method: None,
        endpoint: None,
        demo: None,
        body: include_str!("../../web/docs/category-types.md"),
    },
    Doc {
        slug: "enchantment-types",
        title: "Enchantment Types",
        group: "Types",
        summary: "Enchantment levels 0 to 4 and their rarity names.",
        method: None,
        endpoint: None,
        demo: None,
        body: include_str!("../../web/docs/enchantment-types.md"),
    },
];

pub const GROUPS: &[&str] = &["Getting started", "Endpoints", "Types"];

/// Old documentation URLs from openalbion.com, mapped to their new home.
pub const LEGACY_REDIRECTS: &[(&str, &str)] = &[
    ("/categories", "/docs/categories"),
    ("/weapons", "/docs/weapons"),
    ("/weapon-stats", "/docs/weapon-stats"),
    ("/armors", "/docs/armors"),
    ("/armor-stats", "/docs/armor-stats"),
    ("/accessories", "/docs/accessories"),
    ("/accessories-stats", "/docs/accessories-stats"),
    ("/consumables", "/docs/consumables"),
    ("/consumable-stats", "/docs/consumable-stats"),
    ("/consumable-craftings", "/docs/consumable-craftings"),
    ("/spells", "/docs/spells"),
    ("/category-types", "/docs/category-types"),
    ("/enchantment-types", "/docs/enchantment-types"),
    // VitePress also exposed `.html` URLs; keep those inbound links alive too.
    ("/index.html", "/docs"),
    ("/categories.html", "/docs/categories"),
    ("/weapons.html", "/docs/weapons"),
    ("/weapon-stats.html", "/docs/weapon-stats"),
    ("/armors.html", "/docs/armors"),
    ("/armor-stats.html", "/docs/armor-stats"),
    ("/accessories.html", "/docs/accessories"),
    ("/accessories-stats.html", "/docs/accessories-stats"),
    ("/consumables.html", "/docs/consumables"),
    ("/consumable-stats.html", "/docs/consumable-stats"),
    ("/consumable-craftings.html", "/docs/consumable-craftings"),
    ("/spells.html", "/docs/spells"),
    ("/category-types.html", "/docs/category-types"),
    ("/enchantment-types.html", "/docs/enchantment-types"),
];

pub fn find(slug: &str) -> Option<&'static Doc> {
    DOCS.iter().find(|doc| doc.slug == slug)
}

pub fn path(doc: &Doc) -> String {
    if doc.slug == "introduction" {
        "/docs".to_string()
    } else {
        format!("/docs/{}", doc.slug)
    }
}

pub fn index_of(doc: &Doc) -> usize {
    DOCS.iter()
        .position(|candidate| candidate.slug == doc.slug)
        .unwrap_or(0)
}

/// Renders a docs page, including sidebar, article, table of contents and
/// previous/next navigation.
pub fn render_page(doc: &Doc) -> String {
    let body = strip_leading_h1(doc.body);
    let rendered = markdown::render(&body);
    let active_slug = doc.slug;

    let endpoint_card = endpoint_card(doc);
    let toc = toc_html(&rendered.headings);
    let pager = pager_html(doc);
    let edit_url = format!(
        "{}/blob/main/rust/web/docs/{}.md",
        layout::GITHUB_URL,
        doc.slug
    );

    let content = format!(
        r#"<div class="docs-shell container">
    <details class="docs-sidebar" open>
        <summary class="docs-sidebar-summary">Documentation menu</summary>
        {sidebar}
    </details>

    <div class="docs-main">
        <nav class="breadcrumbs" aria-label="Breadcrumb">
            <a href="/docs">Docs</a>
            <span aria-hidden="true">/</span>
            <span>{title}</span>
        </nav>

        <header class="doc-header">
            <p class="doc-kicker">OpenAlbion docs &middot; {group}</p>
            <h1>{title}</h1>
            <p class="doc-summary">{summary}</p>
            {endpoint_card}
            <p class="doc-edit"><a href="{edit_url}" rel="noopener">Edit this page on GitHub</a></p>
        </header>

        <article class="doc-body">
{body}
        </article>

        {pager}
    </div>

    {toc}
</div>"#,
        sidebar = sidebar(active_slug),
        title = layout::escape(doc.title),
        group = layout::escape(doc.group),
        summary = layout::escape(doc.summary),
        endpoint_card = endpoint_card,
        edit_url = edit_url,
        body = rendered.html,
        pager = pager,
        toc = toc,
    );

    let page_title = format!("{} | OpenAlbion Docs", doc.title);
    layout::page(PageMeta {
        title: &page_title,
        description: doc.summary,
        path: &path(doc),
        active: Active::Docs,
        body_class: "page-docs",
        content,
    })
}

fn strip_leading_h1(markdown: &str) -> String {
    let trimmed = markdown.trim_start();
    if let Some(rest) = trimmed.strip_prefix("# ") {
        if let Some(newline) = rest.find('\n') {
            return rest[newline + 1..].trim_start().to_string();
        }
        return String::new();
    }
    trimmed.to_string()
}

fn endpoint_card(doc: &Doc) -> String {
    let Some(endpoint) = doc.endpoint else {
        return String::new();
    };
    let method = doc.method.unwrap_or("GET");
    let open = doc.demo.map_or(String::new(), |demo| {
        format!(
            r##"<a class="text-link" href="{demo}" target="_blank" rel="noopener">Open example JSON <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><use href="#i-arrow"></use></svg></a>"##,
            demo = layout::escape(demo)
        )
    });

    format!(
        r##"<div class="endpoint-card">
                <span class="method-badge">{method}</span>
                <code class="endpoint-path">{endpoint}</code>
                <div class="endpoint-actions">
                    <button class="copy-btn" type="button" data-copy="{endpoint}" aria-label="Copy endpoint path">
                        <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><use href="#i-copy"></use></svg>
                        <span>Copy</span>
                    </button>
                    {open}
                </div>
            </div>"##,
        method = method,
        endpoint = layout::escape(endpoint),
        open = open,
    )
}

fn doc_search_terms() -> &'static Vec<(&'static str, String)> {
    static INDEX: OnceLock<Vec<(&'static str, String)>> = OnceLock::new();
    INDEX.get_or_init(|| {
        DOCS.iter()
            .map(|doc| {
                let mut terms =
                    format!("{} {} {}", doc.title, doc.summary, doc.group).to_lowercase();
                let rendered = markdown::render(&strip_leading_h1(doc.body));
                for heading in &rendered.headings {
                    terms.push(' ');
                    terms.push_str(&heading.text.to_lowercase());
                }
                (doc.slug, terms)
            })
            .collect()
    })
}

fn sidebar(active_slug: &str) -> String {
    let mut html = String::from(
        r#"<div class="docs-sidebar-inner">
            <label class="docs-filter-label" for="doc-search">Filter docs</label>
            <input class="docs-filter" id="doc-search" type="search" placeholder="Search pages" autocomplete="off" spellcheck="false">
"#,
    );

    for group in GROUPS {
        let pages: Vec<&Doc> = DOCS.iter().filter(|doc| doc.group == *group).collect();
        if pages.is_empty() {
            continue;
        }
        html.push_str("<p class=\"docs-nav-title\">");
        html.push_str(&layout::escape(group));
        html.push_str("</p>\n<ul class=\"docs-nav-list\">\n");
        for doc in pages {
            let current = if doc.slug == active_slug {
                " aria-current=\"page\""
            } else {
                ""
            };
            let terms = doc_search_terms()
                .iter()
                .find(|(slug, _)| *slug == doc.slug)
                .map(|(_, terms)| terms.as_str())
                .unwrap_or_default();
            html.push_str(&format!(
                "<li><a class=\"docs-nav-link\" href=\"{href}\" data-search=\"{search}\"{current}>{title}</a></li>\n",
                href = path(doc),
                search = layout::escape(terms),
                current = current,
                title = layout::escape(doc.title),
            ));
        }
        html.push_str("</ul>\n");
    }

    html.push_str(
        r#"<p class="docs-filter-empty" hidden>No pages match that search.</p>
        </div>"#,
    );
    html
}

fn toc_html(headings: &[Heading]) -> String {
    let items: Vec<&Heading> = headings
        .iter()
        .filter(|heading| (2..=3).contains(&heading.level))
        .collect();
    if items.is_empty() {
        return String::new();
    }

    let mut html = String::from(
        r##"<aside class="docs-toc" aria-label="On this page">
        <p class="docs-toc-title"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><use href="#i-book"></use></svg><span>On this page</span></p>
        <ul>
"##,
    );
    for heading in items {
        html.push_str(&format!(
            "<li class=\"toc-level-{level}\"><a href=\"#{id}\">{text}</a></li>\n",
            level = heading.level,
            id = layout::escape(&heading.id),
            text = layout::escape(&heading.text),
        ));
    }
    html.push_str("</ul>\n    </aside>");
    html
}

fn pager_html(doc: &Doc) -> String {
    let index = index_of(doc);
    let previous = index.checked_sub(1).map(|i| &DOCS[i]);
    let next = DOCS.get(index + 1);

    let mut html = String::from(
        r#"<nav class="doc-pager" aria-label="Documentation pages">
"#,
    );
    if let Some(previous) = previous {
        html.push_str(&format!(
            r#"<a class="pager-link pager-prev" href="{href}">
                <span class="pager-label">Previous</span>
                <span class="pager-title">{title}</span>
            </a>
"#,
            href = path(previous),
            title = layout::escape(previous.title),
        ));
    }
    if let Some(next) = next {
        html.push_str(&format!(
            r#"<a class="pager-link pager-next" href="{href}">
                <span class="pager-label">Next</span>
                <span class="pager-title">{title}</span>
            </a>
"#,
            href = path(next),
            title = layout::escape(next.title),
        ));
    }
    html.push_str("</nav>");
    html
}

/// Documentation cards for the home page.
pub fn cards_html() -> String {
    let mut html = String::new();
    for group in GROUPS {
        let pages: Vec<&Doc> = DOCS.iter().filter(|doc| doc.group == *group).collect();
        if pages.is_empty() {
            continue;
        }
        html.push_str(&format!(
            "<div class=\"docs-group\"><p class=\"docs-group-title\">{}</p><div class=\"docs-cards\">",
            layout::escape(group)
        ));
        for doc in pages {
            html.push_str(&format!(
                r##"<a class="doc-card" href="{href}">
                    <span class="doc-card-title">{title}</span>
                    <span class="doc-card-summary">{summary}</span>
                    <span class="doc-card-arrow" aria-hidden="true"><svg class="icon" viewBox="0 0 24 24"><use href="#i-arrow"></use></svg></span>
                </a>"##,
                href = path(doc),
                title = layout::escape(doc.title),
                summary = layout::escape(doc.summary),
            ));
        }
        html.push_str("</div></div>");
    }
    html
}

/// Endpoint rows for the home page.
pub fn endpoint_menu_html() -> String {
    let mut html = String::new();
    for doc in DOCS.iter().filter(|doc| doc.endpoint.is_some()) {
        html.push_str(&format!(
            r##"<a class="endpoint-row" href="{href}">
                <span class="method-badge">{method}</span>
                <span class="endpoint-path">{endpoint}</span>
                <span class="endpoint-desc">{summary}</span>
                <span class="endpoint-arrow" aria-hidden="true"><svg class="icon" viewBox="0 0 24 24"><use href="#i-arrow"></use></svg></span>
            </a>"##,
            href = path(doc),
            method = doc.method.unwrap_or("GET"),
            endpoint = layout::escape(doc.endpoint.unwrap_or_default()),
            summary = layout::escape(doc.summary),
        ));
    }
    html
}

pub fn sitemap_xml() -> String {
    let mut urls: Vec<String> = vec!["/".to_string(), "/docs".to_string()];
    for doc in DOCS.iter().skip(1) {
        urls.push(path(doc));
    }
    urls.push("/terms-of-service".to_string());
    urls.push("/privacy-policy".to_string());

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for url in urls {
        xml.push_str(&format!(
            "  <url><loc>{}{}</loc></url>\n",
            SITE_URL,
            layout::escape(&url)
        ));
    }
    xml.push_str("</urlset>\n");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_consistent() {
        let mut slugs = std::collections::HashSet::new();
        for doc in DOCS {
            assert!(slugs.insert(doc.slug), "duplicate slug {}", doc.slug);
            assert!(!doc.body.trim().is_empty(), "empty body for {}", doc.slug);
            assert!(GROUPS.contains(&doc.group), "unknown group {}", doc.group);
            if let Some(endpoint) = doc.endpoint {
                assert!(endpoint.starts_with("/api/"), "bad endpoint {}", endpoint);
            }
            if let Some(demo) = doc.demo {
                assert!(demo.starts_with("/api/"), "bad demo {}", demo);
            }
        }
        assert_eq!(DOCS[0].slug, "introduction");
        assert_eq!(find("weapons").map(|doc| doc.title), Some("Weapons"));
    }

    #[test]
    fn legacy_redirects_have_targets() {
        for (from, to) in LEGACY_REDIRECTS {
            assert!(from.starts_with('/'), "bad legacy path {from}");
            assert!(to.starts_with("/docs"), "bad docs target {to}");
        }
    }

    #[test]
    fn docs_pages_render_sidebar_and_content() {
        let html = render_page(find("weapons").unwrap());
        assert!(html.contains("Weapons | OpenAlbion Docs"));
        assert!(html.contains("docs-sidebar"));
        assert!(html.contains("/api/v3/weapons"));
        assert!(html.contains("id=\"get-all-weapons\""));
        assert!(!html.contains("<img src=\"http"));
        assert!(!html.contains("{{"));
    }

    #[test]
    fn sitemap_lists_docs_and_legal_pages() {
        let sitemap = sitemap_xml();
        assert!(sitemap.contains("<loc>https://api.openalbion.com/docs/weapons</loc>"));
        assert!(sitemap.contains("<loc>https://api.openalbion.com/terms-of-service</loc>"));
    }
}
