//! Web pages: the API showcase home page, the merged documentation, the
//! legal pages, and the small public files (`site.js`, CSS, favicon, robots,
//! sitemap). Everything is embedded in the binary; no files are read at
//! runtime.

pub mod docs;
pub mod layout;
pub mod markdown;

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Router;

use crate::state::AppState;

pub use layout::PLAY_STORE_URL;

pub fn web_router(state: Arc<AppState>) -> Router {
    let mut router = Router::new()
        .route("/", get(home))
        .route("/docs", get(docs_index))
        .route(
            "/docs/index",
            get(|| async { Redirect::permanent("/docs") }),
        )
        .route(
            "/docs/introduction",
            get(|| async { Redirect::permanent("/docs") }),
        )
        .route(
            "/introduction",
            get(|| async { Redirect::permanent("/docs") }),
        )
        .route("/docs/{slug}", get(docs_page))
        .route("/privacy-policy", get(privacy_policy))
        .route("/terms-of-service", get(terms_of_service))
        .route("/download", get(download))
        .route("/robots.txt", get(robots))
        .route("/sitemap.xml", get(sitemap))
        .route("/app.css", get(app_css))
        .route("/site.js", get(site_js))
        .route("/logo.png", get(logo))
        .route("/favicon.ico", get(favicon));

    // Legacy documentation URLs from openalbion.com.
    for (from, to) in docs::LEGACY_REDIRECTS {
        let target = *to;
        router = router.route(
            from,
            get(move || async move { Redirect::permanent(target) }),
        );
    }

    router.fallback(not_found).with_state(state)
}

async fn home(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut content = include_str!("../web/pages/home.html").to_string();
    let recipes: usize = state.consumable_craftings.values().map(Vec::len).sum();
    let counts = [
        ("{{COUNT_WEAPONS}}", state.weapons.len()),
        ("{{COUNT_ARMORS}}", state.armors.len()),
        ("{{COUNT_ACCESSORIES}}", state.accessories.len()),
        ("{{COUNT_CONSUMABLES}}", state.consumables.len()),
        ("{{COUNT_SPELLS}}", state.spells.len()),
        ("{{COUNT_CATEGORIES}}", state.categories.len()),
        ("{{COUNT_RECIPES}}", recipes),
    ];
    for (token, value) in counts {
        content = content.replace(token, &value.to_string());
    }
    content = content.replace("{{DOCS_CARDS}}", &docs::cards_html());
    content = content.replace("{{ENDPOINT_MENU}}", &docs::endpoint_menu_html());

    Html(layout::page(layout::PageMeta {
        title: "OpenAlbion — free Albion Online data API",
        description: "Free and open-source Albion Online data API. Weapons, armors, accessories, \
                      consumables, spells, stats, crafting recipes and market prices as clean JSON. \
                      No API key, no sign-up.",
        path: "/",
        active: layout::Active::Home,
        body_class: "page-home",
        content,
    }))
}

async fn docs_index() -> Html<String> {
    Html(docs::render_page(&docs::DOCS[0]))
}

async fn docs_page(Path(slug): Path<String>) -> Response {
    match docs::find(&slug) {
        Some(doc) => Html(docs::render_page(doc)).into_response(),
        None => not_found_response(),
    }
}

async fn privacy_policy() -> Html<String> {
    Html(legal_page(
        "Privacy Policy",
        "/privacy-policy",
        include_str!("../web/pages/privacy.html"),
    ))
}

async fn terms_of_service() -> Html<String> {
    Html(legal_page(
        "Terms of Service",
        "/terms-of-service",
        include_str!("../web/pages/terms.html"),
    ))
}

fn legal_page(title: &'static str, path: &'static str, content: &'static str) -> String {
    let page_title = format!("{title} | OpenAlbion");
    layout::page(layout::PageMeta {
        title: &page_title,
        description:
            "OpenAlbion is a free and open-source platform providing Albion Online data and an API.",
        path,
        active: layout::Active::None,
        body_class: "page-legal",
        content: content.to_string(),
    })
}

async fn not_found() -> Response {
    not_found_response()
}

fn not_found_response() -> Response {
    let content = layout::page(layout::PageMeta {
        title: "Page not found | OpenAlbion",
        description:
            "That page does not exist. Browse the OpenAlbion API documentation or head back home.",
        path: "",
        active: layout::Active::None,
        body_class: "page-not-found",
        content: include_str!("../web/pages/not-found.html").to_string(),
    });
    (StatusCode::NOT_FOUND, Html(content)).into_response()
}

async fn download() -> Response {
    // Laravel's `redirect()->away(...)` responds with a 302.
    (
        StatusCode::FOUND,
        [(header::LOCATION, HeaderValue::from_static(PLAY_STORE_URL))],
    )
        .into_response()
}

async fn robots() -> Response {
    static_text(
        include_str!("../web/robots.txt"),
        "text/plain; charset=utf-8",
    )
}

async fn sitemap() -> Response {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml; charset=utf-8"),
        )],
        docs::sitemap_xml(),
    )
        .into_response()
}

async fn app_css() -> Response {
    static_text(include_str!("../web/app.css"), "text/css; charset=utf-8")
}

async fn site_js() -> Response {
    static_text(
        include_str!("../web/site.js"),
        "text/javascript; charset=utf-8",
    )
}

async fn logo() -> Response {
    let bytes = include_bytes!("../web/logo.png");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, HeaderValue::from_static("image/png"))],
        bytes,
    )
        .into_response()
}

async fn favicon() -> Response {
    let bytes = include_bytes!("../web/favicon.ico");
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("image/x-icon"),
        )],
        bytes,
    )
        .into_response()
}

fn static_text(body: &'static str, content_type: &'static str) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, HeaderValue::from_static(content_type))],
        body,
    )
        .into_response()
}
