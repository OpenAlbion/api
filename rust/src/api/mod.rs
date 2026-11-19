//! API router assembly and middleware.

pub mod handlers;
pub mod weaponry;

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use tower_http::cors::CorsLayer;

use crate::dto::MessageDto;
use crate::state::AppState;

use handlers::{ItemKind, Version};

/// Builds one version's route set (the Laravel v1/v2/v3 route files).
/// `weaponry` marks the sets nested under `/api/weaponry{,V2}` where
/// category resources gain the `path` field.
pub fn version_routes(version: Version, weaponry: bool) -> Router<Arc<AppState>> {
    let routes = Router::new()
        .route(
            "/categories",
            get(move |State(st): State<Arc<AppState>>, Query(q): Query<handlers::CategoryQuery>| async move {
                handlers::categories_index(&st, version, weaponry, &q)
            }),
        )
        .route(
            "/weapons",
            get(move |State(st): State<Arc<AppState>>, Query(q): Query<handlers::ItemQuery>| async move {
                handlers::weapons_index(&st, version, &q)
            }),
        )
        .route(
            "/armors",
            get(move |State(st): State<Arc<AppState>>, Query(q): Query<handlers::ItemQuery>| async move {
                handlers::armors_index(&st, version, &q)
            }),
        )
        .route(
            "/accessories",
            get(move |State(st): State<Arc<AppState>>, Query(q): Query<handlers::ItemQuery>| async move {
                handlers::accessories_index(&st, version, &q)
            }),
        )
        .route(
            "/spells/weapon/{weapon_id}",
            get(move |State(st): State<Arc<AppState>>, PathString(id): PathString| async move {
                handlers::spells_by_item(&st, ItemKind::Weapon, &id)
            }),
        )
        .route(
            "/spells/armor/{armor_id}",
            get(move |State(st): State<Arc<AppState>>, PathString(id): PathString| async move {
                handlers::spells_by_item(&st, ItemKind::Armor, &id)
            }),
        )
        .route(
            "/spells/accessory/{accessory_id}",
            get(move |State(st): State<Arc<AppState>>, PathString(id): PathString| async move {
                handlers::spells_by_item(&st, ItemKind::Accessory, &id)
            }),
        )
        .route(
            "/weapon-stats/weapon/{weapon_id}",
            get(move |State(st): State<Arc<AppState>>, PathString(id): PathString| async move {
                handlers::weapon_stats_by(&st, &id)
            }),
        )
        .route(
            "/armor-stats/armor/{armor_id}",
            get(move |State(st): State<Arc<AppState>>, PathString(id): PathString| async move {
                handlers::armor_stats_by(&st, &id)
            }),
        )
        .route(
            "/accessory-stats/accessory/{accessory_id}",
            get(move |State(st): State<Arc<AppState>>, PathString(id): PathString| async move {
                handlers::accessory_stats_by(&st, &id)
            }),
        );

    // Consumables exist from v2 on.
    if version == Version::V1 {
        routes
    } else {
        routes
            .route(
                "/consumables",
                get(move |State(st): State<Arc<AppState>>, Query(q): Query<handlers::ItemQuery>| async move {
                    handlers::consumables_index(&st, version, &q)
                }),
            )
            .route(
                "/consumable-stats/consumable/{consumable_id}",
                get(move |State(st): State<Arc<AppState>>, PathString(id): PathString| async move {
                    handlers::consumable_stats_by(&st, &id)
                }),
            )
            .route(
                "/consumable-craftings/consumable/{consumable_id}",
                get(move |State(st): State<Arc<AppState>>, PathString(id): PathString| async move {
                    handlers::craftings_by(&st, &id)
                }),
            )
    }
}

/// axum's `Path<String>` rejects non-UTF8 segments with a 400; the Laravel
/// app accepts any string and simply matches nothing. This tiny extractor
/// keeps the parity (a lossy-UTF8 decode still yields a non-numeric id,
/// which results in an empty `data` list).
struct PathString(pub String);

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for PathString {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let raw = parts.uri.path().rsplit('/').next().unwrap_or_default();
        Ok(PathString(raw.to_string()))
    }
}

/// The `/api/weaponry` (V1 set) and `/api/weaponryV2` (V2 set) routers:
/// the four app endpoints plus the nested version routes. Fully public —
/// the Laravel `X-WEAPONRY-KEY` check was dropped on purpose.
fn weaponry_routes(version: Version) -> Router<Arc<AppState>> {
    let v2 = version == Version::V2;

    version_routes(version, true)
        .route(
            "/aod/{region}/item/{item_id}/price",
            get(weaponry::aod_price),
        )
        .route(
            "/search",
            get(
                move |State(st): State<Arc<AppState>>, Query(q): Query<weaponry::SearchQuery>| async move {
                    if v2 {
                        weaponry::search_v2(&st, &q)
                    } else {
                        weaponry::search_v1(&st, &q)
                    }
                },
            ),
        )
        .route("/bug-report", post(weaponry::bug_report))
        .route("/version-check", get(weaponry::version_check))
}

/// The `/api` router. On the server an in-process per-IP limiter runs as
/// middleware; on Workers (no `server` feature) rate limiting is done by the
/// Workers Rate Limiting binding before the request reaches the router.
pub fn api_router(state: Arc<AppState>) -> Router {
    let router = Router::new()
        .nest("/v1", version_routes(Version::V1, false))
        .nest("/v2", version_routes(Version::V2, false))
        .nest("/v3", version_routes(Version::V3, false))
        .nest("/weaponry", weaponry_routes(Version::V1))
        .nest("/weaponryV2", weaponry_routes(Version::V2))
        .fallback(api_fallback);

    #[cfg(feature = "server")]
    let router = router.layer(axum::middleware::from_fn_with_state(state.clone(), rate_limit_mw));

    router.layer(CorsLayer::very_permissive()).with_state(state)
}

/// The 429 response (Laravel's throttle shape).
pub fn too_many_attempts(retry_after: u64, limit: u64) -> Response {
    let mut response = Json(MessageDto::new("Too Many Attempts.")).into_response();
    *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
    response
        .headers_mut()
        .insert(header::RETRY_AFTER, HeaderValue::from(retry_after));
    response
        .headers_mut()
        .insert("x-ratelimit-limit", HeaderValue::from(limit));
    response
}

#[cfg(feature = "server")]
async fn rate_limit_mw(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let ip = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if let Some(retry_after) = state.rate_limit.hit(&ip, crate::platform::now_unix()) {
        return too_many_attempts(retry_after, state.rate_limit.limit);
    }

    next.run(req).await
}

async fn api_fallback() -> Response {
    (StatusCode::NOT_FOUND, Json(MessageDto::new("Not Found"))).into_response()
}
