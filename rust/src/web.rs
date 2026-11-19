//! Web pages: home, terms-of-service, privacy-policy (embedded HTML), plus
//! the small public files (`/download`, robots.txt, favicon.ico).

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;

pub const PLAY_STORE_URL: &str = "https://play.google.com/store/apps/details?id=com.openalbion.weaponry";

pub fn web_router() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/privacy-policy", get(privacy_policy))
        .route("/terms-of-service", get(terms_of_service))
        .route("/download", get(download))
        .route("/robots.txt", get(robots))
        .route("/app.css", get(app_css))
        .route("/favicon.ico", get(favicon))
}

async fn home() -> Html<&'static str> {
    Html(include_str!("../web/home.html"))
}

async fn privacy_policy() -> Html<&'static str> {
    Html(include_str!("../web/privacy-policy.html"))
}

async fn terms_of_service() -> Html<&'static str> {
    Html(include_str!("../web/terms-of-service.html"))
}

async fn app_css() -> Response {
    let css = include_str!("../web/app.css");
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/css; charset=utf-8"),
        )],
        css,
    )
        .into_response()
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
    let body = include_str!("../web/robots.txt");
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        )],
        body,
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
