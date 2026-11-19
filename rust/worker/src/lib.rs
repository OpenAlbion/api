//! Cloudflare Workers entry point for the OpenAlbion API.
//!
//! Runs the exact same axum router as the native server (`../src`), compiled
//! to WebAssembly. The dataset is embedded in the wasm binary and parsed
//! once per isolate, on the first request.

use std::borrow::Cow;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use axum::Router;
use openalbion_api::{api, web, AppConfig, AppState};
use tower::{Layer, Service};
use tower_http::normalize_path::{NormalizePath, NormalizePathLayer};
use worker::{event, Context, Env, HttpRequest};

type App = NormalizePath<Router>;

static APP: OnceLock<App> = OnceLock::new();

/// Must match `simple.limit` of the `API_RATE_LIMITER` binding in
/// wrangler.toml (only used for the `x-ratelimit-limit` header).
const RATE_LIMIT_PER_MINUTE: u64 = 60;

macro_rules! tables {
    ($($name:literal),* $(,)?) => {
        fn embedded_table(table: &str) -> std::io::Result<Cow<'static, [u8]>> {
            match table {
                $($name => Ok(Cow::Borrowed(include_bytes!(concat!("../../data/", $name, ".json")))),)*
                other => Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("table {other} is not embedded"),
                )),
            }
        }
    };
}

tables!(
    "categories",
    "weapons",
    "armors",
    "accessories",
    "consumables",
    "spells",
    "weapon_spells",
    "armor_spells",
    "accessory_spells",
    "weapon_stats",
    "armor_stats",
    "accessory_stats",
    "consumable_stats",
    "consumable_craftings",
);

fn env_string(env: &Env, key: &str) -> Option<String> {
    env.secret(key)
        .map(|s| s.to_string())
        .or_else(|_| env.var(key).map(|v| v.to_string()))
        .ok()
        .filter(|s| !s.is_empty())
}

fn build_app(env: &Env) -> App {
    let defaults = AppConfig::default();
    let config = AppConfig {
        slack_webhook_url: env_string(env, "SLACK_WEBHOOK_URL"),
        rate_limit_per_minute: RATE_LIMIT_PER_MINUTE,
        aod_cache_ttl: env_string(env, "AOD_CACHE_SECONDS")
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(defaults.aod_cache_ttl),
        aod_url_template: env_string(env, "AOD_URL_TEMPLATE").unwrap_or(defaults.aod_url_template),
    };

    let state = Arc::new(
        AppState::from_loader(&embedded_table, config).expect("embedded dataset is valid"),
    );
    let router = Router::new()
        .nest("/api", api::api_router(state.clone()))
        .merge(web::web_router(state));
    NormalizePathLayer::trim_trailing_slash().layer(router)
}

/// Per-IP limit via the Workers Rate Limiting binding. Fails open if the
/// binding is missing or errors.
async fn over_rate_limit(req: &HttpRequest, env: &Env) -> bool {
    let Ok(limiter) = env.rate_limiter("API_RATE_LIMITER") else {
        return false;
    };
    let ip = req
        .headers()
        .get("cf-connecting-ip")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    matches!(limiter.limit(ip).await, Ok(outcome) if !outcome.success)
}

#[event(start)]
fn start() {
    console_error_panic_hook_lite();
}

/// Route Rust panics to `console.error` so they show up in Workers logs.
fn console_error_panic_hook_lite() {
    std::panic::set_hook(Box::new(|info| {
        worker::console_error!("panic: {info}");
    }));
}

#[event(fetch)]
async fn fetch(req: HttpRequest, env: Env, _ctx: Context) -> worker::Result<axum::response::Response> {
    let path = req.uri().path();
    if (path == "/api" || path.starts_with("/api/")) && over_rate_limit(&req, &env).await {
        return Ok(api::too_many_attempts(60, RATE_LIMIT_PER_MINUTE));
    }

    let mut app = APP.get_or_init(|| build_app(&env)).clone();
    let response = match app.call(req).await {
        Ok(response) => response,
        Err(never) => match never {},
    };
    Ok(response)
}
