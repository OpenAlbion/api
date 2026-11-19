//! OpenAlbion API — Rust rewrite.
//!
//! Serves the v1/v2/v3/weaponry/weaponryV2 API endpoints plus the home,
//! terms-of-service and privacy-policy pages, reading the dataset from
//! JSON files instead of a database.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use tower::Layer;
use tracing_subscriber::EnvFilter;

use openalbion_api::{state::check_data_dir, web, AppConfig, AppState};

/// Loads `KEY=VALUE` lines from a local `.env` (if present) without
/// overriding variables that are already set in the environment.
fn load_dot_env() {
    let Ok(content) = std::fs::read_to_string(".env") else {
        return;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let mut value = value.trim().to_string();
        // Strip matching quotes.
        if value.len() >= 2
            && ((value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\'')))
        {
            value = value[1..value.len() - 1].to_string();
        }
        if std::env::var_os(key).is_none() {
            std::env::set_var(key, value);
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    load_dot_env();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let host: std::net::IpAddr = env_or("HOST", "0.0.0.0")
        .parse()
        .map_err(|e| std::io::Error::other(format!("invalid HOST: {e}")))?;
    let port: u16 = env_or("PORT", "8080")
        .parse()
        .map_err(|e| std::io::Error::other(format!("invalid PORT: {e}")))?;
    let data_dir = PathBuf::from(env_or("DATA_DIR", "data"));
    let rate_limit_per_minute: u64 = env_or("RATE_LIMIT_PER_MINUTE", "60")
        .parse()
        .map_err(|e| std::io::Error::other(format!("invalid RATE_LIMIT_PER_MINUTE: {e}")))?;
    let aod_cache_seconds: u64 = env_or("AOD_CACHE_SECONDS", "180")
        .parse()
        .map_err(|e| std::io::Error::other(format!("invalid AOD_CACHE_SECONDS: {e}")))?;

    check_data_dir(&data_dir)?;

    let config = AppConfig {
        slack_webhook_url: std::env::var("SLACK_WEBHOOK_URL").ok().filter(|k| !k.is_empty()),
        rate_limit_per_minute,
        aod_cache_ttl: Duration::from_secs(aod_cache_seconds),
        aod_url_template: match std::env::var("AOD_URL_TEMPLATE") {
            Ok(t) if !t.is_empty() => t,
            _ => AppConfig::default().aod_url_template,
        },
    };

    let state = Arc::new(AppState::load(&data_dir, config)?);

    let app: Router = Router::new()
        .nest("/api", openalbion_api::api::api_router(state.clone()))
        .merge(web::web_router(state));

    // Trailing-slash tolerance (Laravel matches `/api/v1/weapons/` as well
    // as `/api/v1/weapons`). NormalizePathLayer must wrap the router before
    // route matching, so it cannot be a regular `Router::layer`.
    let app: tower_http::normalize_path::NormalizePath<Router> =
        tower_http::normalize_path::NormalizePathLayer::trim_trailing_slash().layer(app);

    let addr = SocketAddr::from((host, port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("OpenAlbion API listening on http://{addr}");

    axum::serve(
        listener,
        axum::ServiceExt::<axum::extract::Request>::into_make_service_with_connect_info::<SocketAddr>(
            app,
        ),
    )
    .await?;

    Ok(())
}
