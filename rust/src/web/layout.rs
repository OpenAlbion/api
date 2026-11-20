//! Shared page shell: document head, site header, footer and the embedded
//! OpenAlbion logo plus inline SVG artwork. No webfonts or third-party assets
//! are referenced.

pub use crate::web::markdown::escape_html as escape;

use crate::web::markdown::escape_html;

pub const SITE_URL: &str = "https://api.openalbion.com";
pub const SITE_NAME: &str = "OpenAlbion";
pub const PLAY_STORE_URL: &str =
    "https://play.google.com/store/apps/details?id=com.openalbion.weaponry";
pub const GITHUB_URL: &str = "https://github.com/OpenAlbion/api";
/// Community project built on the API; linked from the footer and the home page.
pub const ARMORY_URL: &str = "https://armory.openalbion.com";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Active {
    Home,
    Docs,
    None,
}

pub struct PageMeta<'a> {
    /// Full `<title>` text, already including the site suffix.
    pub title: &'a str,
    pub description: &'a str,
    /// Canonical path beginning with `/`. Empty disables the canonical tag.
    pub path: &'a str,
    pub active: Active,
    pub body_class: &'a str,
    pub content: String,
}

/// Renders a complete HTML document around the supplied page content.
pub fn page(meta: PageMeta<'_>) -> String {
    let title = escape_html(meta.title);
    let description = escape_html(meta.description);
    let canonical = if meta.path.is_empty() {
        String::new()
    } else {
        let url = format!("{}{}", SITE_URL, meta.path);
        format!(
            "<link rel=\"canonical\" href=\"{url}\">\n    <meta property=\"og:url\" content=\"{url}\">\n",
            url = escape_html(&url)
        )
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">

    <title>{title}</title>
    <meta name="description" content="{description}">
    <meta name="theme-color" content="#e84552">

    <meta property="og:site_name" content="{site}">
    <meta property="og:type" content="website">
    <meta property="og:title" content="{title}">
    <meta property="og:description" content="{description}">
    <meta name="twitter:card" content="summary">
    <meta name="twitter:title" content="{title}">
    <meta name="twitter:description" content="{description}">
    {canonical}
    <link rel="icon" href="/logo.png" type="image/png">
    <link rel="icon" href="/favicon.ico" sizes="any">
    <link rel="stylesheet" href="/app.css">
    <script src="/site.js" defer></script>
</head>
<body class="{body_class}">
    <a class="skip-link" href="#main">Skip to content</a>
    {sprite}
    {header}
    <main id="main">
{content}
    </main>
    {footer}
</body>
</html>
"##,
        title = title,
        description = description,
        site = SITE_NAME,
        canonical = canonical,
        body_class = escape_html(meta.body_class),
        sprite = sprite(),
        header = header(meta.active),
        content = meta.content,
        footer = footer(),
    )
}

fn header(active: Active) -> String {
    let docs_current = if active == Active::Docs {
        " aria-current=\"page\""
    } else {
        ""
    };

    format!(
        r##"<header class="site-header">
        <div class="container header-inner">
            <a class="brand" href="/" aria-label="OpenAlbion home">
                <img class="brand-logo" src="/logo.png" width="42" height="42" alt="">
                <span class="brand-name">OpenAlbion</span>
            </a>

            <button class="nav-toggle" type="button" aria-expanded="false" aria-controls="site-nav">
                Menu
            </button>

            <nav class="site-nav" id="site-nav" aria-label="Main">
                <a class="nav-link" href="/docs"{docs_current}>Docs</a>
                <a class="nav-link" href="/#playground">Playground</a>
                <a class="nav-link" href="{github}" rel="noopener">GitHub</a>
                <a class="nav-demo nav-demo-web" href="{armory}" rel="noopener">Web demo</a>
                <a class="nav-demo nav-demo-android" href="/download">Android demo</a>
            </nav>
        </div>
    </header>"##,
        docs_current = docs_current,
        github = GITHUB_URL,
        armory = ARMORY_URL,
    )
}

fn footer() -> String {
    format!(
        r##"<footer class="site-footer">
        <div class="container footer-grid">
            <div class="footer-brand">
                <a class="brand" href="/" aria-label="OpenAlbion home">
                    <img class="brand-logo" src="/logo.png" width="42" height="42" alt="">
                    <span class="brand-name">OpenAlbion</span>
                </a>
                <p>Free and open-source Albion Online data, served as clean JSON from a Rust API.</p>
                <span class="sticker sticker-mint">MIT licensed</span>
            </div>

            <nav class="footer-col" aria-label="API">
                <h2>API</h2>
                <a href="/docs">Documentation</a>
                <a href="/#playground">Playground</a>
                <a href="/sitemap.xml">Sitemap</a>
                <a href="{github}" rel="noopener">GitHub</a>
            </nav>

            <nav class="footer-col" aria-label="Project">
                <h2>Project</h2>
                <a href="{armory}" rel="noopener">Armory &mdash; web demo</a>
                <a href="/download">Weaponry &mdash; Android demo</a>
                <a href="{github}/issues" rel="noopener">Report an issue</a>
                <a href="/terms-of-service">Terms of Service</a>
                <a href="/privacy-policy">Privacy Policy</a>
            </nav>

            <div class="footer-col footer-note">
                <h2>Data</h2>
                <p>Collected from the Albion Online Wiki and the game's public data dumps.</p>
                <p>Not affiliated with Sandbox Interactive.</p>
            </div>
        </div>

        <div class="container footer-bottom">
            <span>&copy; OpenAlbion &mdash; MIT licensed</span>
            <span>Built with Rust &middot; no API key, no sign-up</span>
        </div>
    </footer>"##,
        github = GITHUB_URL,
        armory = ARMORY_URL,
    )
}

/// Original icon artwork used by the pages. Everything here is inline SVG, so the site
/// makes no image requests and every glyph is drawn here.
fn sprite() -> &'static str {
    r#"<svg class="svg-sprite" aria-hidden="true" focusable="false" xmlns="http://www.w3.org/2000/svg">
        <symbol id="i-sparkle" viewBox="0 0 24 24">
            <path d="M12 2.2c.7 4.9 2.9 7.1 7.8 7.8-4.9.7-7.1 2.9-7.8 7.8-.7-4.9-2.9-7.1-7.8-7.8 4.9-.7 7.1-2.9 7.8-7.8Z" fill="currentColor"/>
            <path d="M19.2 16.2c.3 1.9 1.1 2.7 3 3-1.9.3-2.7 1.1-3 3-.3-1.9-1.1-2.7-3-3 1.9-.3 2.7-1.1 3-3Z" fill="currentColor"/>
        </symbol>
        <symbol id="i-sword" viewBox="0 0 24 24">
            <path d="M14.5 3.5 20 3l-.5 5.5-8.7 8.7-4.3-4.3 8-9.4Z"/>
            <path d="m8.2 14.2-2.4 2.4m-1.6 4.2 2.2-2.2m0 0-2.2-2.2m2.2 2.2 2.2 2.2"/>
            <path d="m13.3 4.7 3.9 3.9"/>
        </symbol>
        <symbol id="i-shield" viewBox="0 0 24 24">
            <path d="M12 2.8 20 6v5.6c0 4.3-2.9 7.7-8 9.6-5.1-1.9-8-5.3-8-9.6V6l8-3.2Z"/>
            <path d="M12 7.2c.4 2.5 1.5 3.6 4 4-2.5.4-3.6 1.5-4 4-.4-2.5-1.5-3.6-4-4 2.5-.4 3.6-1.5 4-4Z"/>
        </symbol>
        <symbol id="i-spell" viewBox="0 0 24 24">
            <ellipse cx="12" cy="12" rx="8.5" ry="4.5" transform="rotate(-32 12 12)"/>
            <ellipse cx="12" cy="12" rx="8.5" ry="4.5" transform="rotate(32 12 12)"/>
            <circle cx="12" cy="12" r="2.2"/>
        </symbol>
        <symbol id="i-flask" viewBox="0 0 24 24">
            <path d="M9 3h6M10 3v6.2l-4.7 8.2A2 2 0 0 0 7.1 21h9.8a2 2 0 0 0 1.8-3.6L14 9.2V3"/>
            <path d="M8 15.5h8"/>
            <circle cx="11.3" cy="18" r=".8" fill="currentColor" stroke="none"/>
            <circle cx="14.4" cy="19.1" r=".6" fill="currentColor" stroke="none"/>
        </symbol>
        <symbol id="i-coins" viewBox="0 0 24 24">
            <ellipse cx="9.5" cy="8" rx="5.5" ry="2.6"/>
            <path d="M4 8v4c0 1.4 2.5 2.6 5.5 2.6S15 13.4 15 12V8"/>
            <path d="M9 14.5v3c0 1.4 2.5 2.6 5.5 2.6S20 18.9 20 17.5v-4"/>
            <path d="M14.5 12.5c3 0 5.5 1 5.5 2.2"/>
        </symbol>
        <symbol id="i-search" viewBox="0 0 24 24">
            <circle cx="10.5" cy="10.5" r="6"/>
            <path d="m15 15 5 5"/>
        </symbol>
        <symbol id="i-code" viewBox="0 0 24 24">
            <path d="m8 7-5 5 5 5M16 7l5 5-5 5M13.5 4l-3 16"/>
        </symbol>
        <symbol id="i-book" viewBox="0 0 24 24">
            <path d="M4 4.5A2.5 2.5 0 0 1 6.5 2H20v17.5H6.5A2.5 2.5 0 0 0 4 22V4.5Z"/>
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
            <path d="M8 6h8M8 9.5h5"/>
        </symbol>
        <symbol id="i-arrow" viewBox="0 0 24 24">
            <path d="M4 12h15m-5-5 5 5-5 5"/>
        </symbol>
        <symbol id="i-copy" viewBox="0 0 24 24">
            <rect x="8" y="8" width="12" height="12" rx="2.5"/>
            <path d="M16 8V6.5A2.5 2.5 0 0 0 13.5 4h-7A2.5 2.5 0 0 0 4 6.5v7A2.5 2.5 0 0 0 6.5 16H8"/>
        </symbol>
        <symbol id="i-play" viewBox="0 0 24 24">
            <path d="M8 5.5v13l10-6.5-10-6.5Z" fill="currentColor" stroke="none"/>
        </symbol>
        <symbol id="i-check" viewBox="0 0 24 24">
            <path d="m4.5 12.5 4.5 4.5 10.5-11"/>
        </symbol>
    </svg>"#
}
