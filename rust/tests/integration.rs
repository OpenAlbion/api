//! Router-level integration tests: a tiny fixture dataset, axum `oneshot`
//! requests, and assertions on the exact JSON the Laravel app would emit.

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use tower::{Layer, ServiceExt};

use openalbion_api::{api, state::AppConfig, web, AppState};

// ---------------------------------------------------------------------------
// Fixture dataset
// ---------------------------------------------------------------------------

fn write_table(dir: &std::path::Path, table: &str, rows: Value) {
    std::fs::write(dir.join(format!("{table}.json")), rows.to_string()).unwrap();
}

/// Creates a temp data dir with a small, hand-checkable dataset and returns
/// the loaded `AppState`.
fn fixture_state(dir: &std::path::Path) -> Arc<AppState> {
    write_table(
        dir,
        "categories",
        json!([
            { "id": 1, "name": "Weapons", "parent_id": null, "path": null, "type": "weapon", "version": 1 },
            { "id": 2, "name": "Swords", "parent_id": 1, "path": "/wiki/Swords", "type": "weapon", "version": 1 },
            { "id": 3, "name": "Armors", "parent_id": null, "path": null, "type": "armor", "version": 1 },
            { "id": 4, "name": "Bags", "parent_id": null, "path": null, "type": "accessory", "version": 1 },
            { "id": 5, "name": "Foods", "parent_id": null, "path": null, "type": "consumable", "version": 1 },
            // version 2 root: hidden from v1, visible in v2/v3
            { "id": 6, "name": "Mounts", "parent_id": null, "path": null, "type": "mount", "version": 2 },
        ]),
    );
    write_table(
        dir,
        "weapons",
        json!([
            { "id": 1, "category_id": 1, "subcategory_id": 2, "name": "Adept's Sword", "identifier": "T4_SWORD", "tier": 4, "item_power": 700, "path": "/wiki/Adept's_Sword" },
            { "id": 2, "category_id": 1, "subcategory_id": 2, "name": "Expert's Sword", "identifier": "T5_SWORD", "tier": 5, "item_power": 800, "path": null },
            { "id": 3, "category_id": 1, "subcategory_id": 2, "name": "Sword Zero Tier", "identifier": "T0_SWORD", "tier": 0, "item_power": 1, "path": null }
        ]),
    );
    write_table(
        dir,
        "armors",
        json!([
            { "id": 1, "category_id": 3, "subcategory_id": null, "name": "Adept's Plate Armor", "identifier": "T4_PLATE_ARMOR", "tier": 4, "item_power": 700, "path": null }
        ]),
    );
    write_table(
        dir,
        "accessories",
        json!([
            { "id": 1, "category_id": 4, "subcategory_id": null, "name": "Adept's Bag", "identifier": "T4_BAG", "tier": 4, "item_power": 600, "path": null }
        ]),
    );
    write_table(
        dir,
        "consumables",
        json!([
            { "id": 1, "category_id": 5, "subcategory_id": null, "name": "Chicken Omelette", "identifier": "T4_OMELLETTE", "tier": 4, "item_power": 10, "path": null, "info": "Restores health." },
            { "id": 2, "category_id": 5, "subcategory_id": null, "name": "Pork Omelette", "identifier": "T5_OMELLETTE", "tier": 5, "item_power": 12, "path": null, "info": null }
        ]),
    );
    write_table(
        dir,
        "spells",
        json!([
            { "id": 1, "name": "Slash", "identifier": "Slash", "slot": "q", "attributes": [{"name": "Energy Cost", "value": "6"}], "description": "<b>Deal</b> 100 damage", "preview": "https://example.com/slash.png", "ref": "SwordSlash" },
            { "id": 2, "name": "Cleave", "identifier": "Cleave", "slot": "w", "attributes": [], "description": "AoE <i>damage</i>", "preview": null, "ref": "SwordCleave" },
            { "id": 3, "name": "Sword Passive", "identifier": "SwordPassive", "slot": "passive", "attributes": [], "description": "Passive", "preview": null, "ref": "SP" }
        ]),
    );
    write_table(
        dir,
        "weapon_spells",
        json!([
            { "id": 1, "weapon_id": 1, "spell_id": 1 },
            { "id": 2, "weapon_id": 1, "spell_id": 2 },
            { "id": 3, "weapon_id": 1, "spell_id": 3 }
        ]),
    );
    write_table(dir, "armor_spells", json!([{ "id": 1, "armor_id": 1, "spell_id": 3 }]));
    write_table(dir, "accessory_spells", json!([{ "id": 1, "accessory_id": 1, "spell_id": 3 }]));
    write_table(
        dir,
        "weapon_stats",
        json!([
            { "id": 1, "weapon_id": 1, "quality": "Normal", "enchantment": 0, "stats": [{"name": "Item Power", "value": "700"}] },
            { "id": 2, "weapon_id": 1, "quality": "Good", "enchantment": 0, "stats": [{"name": "Item Power", "value": "720"}] },
            { "id": 3, "weapon_id": 1, "quality": "Normal", "enchantment": 2, "stats": [{"name": "Item Power", "value": "900"}] }
        ]),
    );
    write_table(dir, "armor_stats", json!([
        { "id": 1, "armor_id": 1, "quality": "Normal", "enchantment": 0, "stats": [{"name": "Item Power", "value": "700"}] }
    ]));
    write_table(dir, "accessory_stats", json!([
        { "id": 1, "accessory_id": 1, "quality": "Normal", "enchantment": 0, "stats": [{"name": "Item Power", "value": "600"}] }
    ]));
    write_table(
        dir,
        "consumable_stats",
        json!([
            { "id": 1, "consumable_id": 1, "quality": "Normal", "enchantment": 0, "stats": [{"name": "Health", "value": "50"}] },
            { "id": 2, "consumable_id": 1, "quality": "Normal", "enchantment": 1, "stats": [{"name": "Health", "value": "75"}] }
        ]),
    );
    write_table(
        dir,
        "consumable_craftings",
        json!([
            { "id": 1, "consumable_id": 1, "per_craft": 1, "enchantment": 0, "requirements": [{"name": "Raw Chicken", "identifier": "T3_MEAT"}] },
            { "id": 2, "consumable_id": 1, "per_craft": 2, "enchantment": 1, "requirements": [{"name": "Raw Chicken", "identifier": "T4_MEAT"}] }
        ]),
    );

    let config = AppConfig {
        rate_limit_per_minute: 1_000_000, // effectively off; dedicated test below
        ..AppConfig::default()
    };
    Arc::new(AppState::load(dir, config).expect("fixture loads"))
}

fn temp_data_dir() -> (std::path::PathBuf, impl Drop) {
    // A monotonic counter keeps names unique even when several tests start in
    // the same clock tick (coarse timer resolution on macOS).
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "openalbion-test-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    struct Guard(std::path::PathBuf);
    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let guard = Guard(dir.clone());
    (dir, guard)
}

/// Builds the full app exactly like `main` does (including the
/// trailing-slash normalization).
fn build_app(state: Arc<AppState>) -> tower_http::normalize_path::NormalizePath<Router> {
    let app: Router = Router::new()
        .nest("/api", api::api_router(state.clone()))
        .merge(web::web_router(state));
    tower_http::normalize_path::NormalizePathLayer::trim_trailing_slash().layer(app)
}

async fn get(app: &mut tower_http::normalize_path::NormalizePath<Router>, uri: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap() };
    (status, value)
}

async fn get_raw(app: &mut tower_http::normalize_path::NormalizePath<Router>, uri: &str) -> axum::response::Response {
    app.clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn get_text(
    app: &mut tower_http::normalize_path::NormalizePath<Router>,
    uri: &str,
) -> (StatusCode, String) {
    let response = get_raw(app, uri).await;
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

async fn post_json(
    app: &mut tower_http::normalize_path::NormalizePath<Router>,
    uri: &str,
    body: Value,
) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap() };
    (status, value)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn categories_respect_version_filters() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    let (status, v1) = get(&mut app, "/api/v1/categories").await;
    assert_eq!(status, StatusCode::OK);
    // v1 hides the version-2 "Mounts" root.
    assert_eq!(v1["data"].as_array().unwrap().len(), 4);
    assert!(v1["data"][0].get("path").is_none(), "path only on weaponry routes");
    assert_eq!(v1["data"][0]["subcategories"][0]["name"], "Swords");

    let (_, v2) = get(&mut app, "/api/v2/categories").await;
    assert_eq!(v2["data"].as_array().unwrap().len(), 5);

    // Type filter
    let (_, weapons_only) = get(&mut app, "/api/v1/categories?type=weapon").await;
    assert_eq!(weapons_only["data"].as_array().unwrap().len(), 1);

    // PHP-falsy "0" means no filter
    let (_, zero) = get(&mut app, "/api/v1/categories?type=0").await;
    assert_eq!(zero["data"].as_array().unwrap().len(), 4);
}

#[tokio::test]
async fn weaponry_routes_are_public_and_gain_path() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    // Public — no key header at all.
    let (status, value) = get(&mut app, "/api/weaponry/categories").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["data"][0]["path"], "weapons");
    assert_eq!(value["data"][0]["subcategories"][0]["path"], "weapons");
    assert_eq!(value["data"][1]["path"], "armors");

    // Same for the V2 set.
    let (status, value) = get(&mut app, "/api/weaponryV2/categories").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["data"][0]["path"], "weapons");

    // ...and for the app endpoints.
    let (status, _) = get(&mut app, "/api/weaponry/version-check").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn item_lists_match_version_shapes() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    let (_, v1) = get(&mut app, "/api/v1/weapons").await;
    assert_eq!(v1["data"].as_array().unwrap().len(), 3);
    assert!(v1["data"][0].get("category").is_none(), "v1 resources are flat");
    // tier is a formatted string, zero becomes null
    assert_eq!(v1["data"][0]["tier"], json!("4.0"));
    assert_eq!(v1["data"][2]["tier"], Value::Null);

    let (_, v3) = get(&mut app, "/api/v3/weapons").await;
    assert_eq!(v3["data"][0]["category"]["name"], "Weapons");
    assert_eq!(v3["data"][0]["subcategory"]["name"], "Swords");
    assert!(v3["data"][0]["category"].get("subcategories").is_none());

    // Filters
    let (_, tier4) = get(&mut app, "/api/v3/weapons?tier=4").await;
    assert_eq!(tier4["data"].as_array().unwrap().len(), 1);
    let (_, falsy) = get(&mut app, "/api/v3/weapons?tier=0").await;
    assert_eq!(falsy["data"].as_array().unwrap().len(), 3);
    let (_, bad) = get(&mut app, "/api/v3/weapons?category_id=abc").await;
    assert_eq!(bad["data"].as_array().unwrap().len(), 0);

    // v3 consumables carry info + nested category
    let (_, cons) = get(&mut app, "/api/v3/consumables").await;
    assert_eq!(cons["data"][0]["info"], "Restores health.");
    assert_eq!(cons["data"][0]["category"]["name"], "Foods");
    assert_eq!(cons["data"][1]["info"], Value::Null);
}

#[tokio::test]
async fn spells_group_by_slot_in_first_seen_order() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    let (status, value) = get(&mut app, "/api/v3/spells/weapon/1").await;
    assert_eq!(status, StatusCode::OK);
    let slots: Vec<&str> = value["data"].as_array().unwrap().iter().map(|g| g["slot"].as_str().unwrap()).collect();
    assert_eq!(slots, vec!["First Slot", "Second Slot", "Passive"]);
    let first = &value["data"][0]["spells"][0];
    assert_eq!(first["name"], "Slash");
    assert_eq!(first["description"], "Deal 100 damage");
    assert_eq!(first["description_html"], "<b>Deal</b> 100 damage");
    assert_eq!(first["icon"], "https://render.albiononline.com/v1/spell/Slash.png?size=217&locale=en");
    assert_eq!(first["attributes"][0]["name"], "Energy Cost");
    assert_eq!(value["data"][1]["spells"][0]["preview"], Value::Null);

    // armor + accessory endpoints
    let (_, armor) = get(&mut app, "/api/v3/spells/armor/1").await;
    assert_eq!(armor["data"][0]["slot"], "Passive");
    let (_, acc) = get(&mut app, "/api/v3/spells/accessory/1").await;
    assert_eq!(acc["data"][0]["spells"][0]["name"], "Sword Passive");

    // unknown item -> empty data; non-numeric id -> empty data
    let (_, empty) = get(&mut app, "/api/v3/spells/weapon/999").await;
    assert_eq!(empty["data"].as_array().unwrap().len(), 0);
    let (_, nonnum) = get(&mut app, "/api/v3/spells/weapon/abc").await;
    assert_eq!(nonnum["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn stats_group_by_enchantment_with_icon() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let app = build_app(state.clone());

    let (status, value) = get(&mut app.clone(), "/api/v3/weapon-stats/weapon/1").await;
    assert_eq!(status, StatusCode::OK);
    let groups = value["data"].as_array().unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0]["enchantment"], 0);
    assert_eq!(groups[0]["stats"].as_array().unwrap().len(), 2);
    assert_eq!(groups[0]["icon"], "https://render.albiononline.com/v1/item/T4_SWORD@0.png?quality=0&size=217&locale=en");
    assert_eq!(groups[1]["enchantment"], 2);
    assert_eq!(groups[1]["icon"], "https://render.albiononline.com/v1/item/T4_SWORD@2.png?quality=0&size=217&locale=en");
    // the nested weapon is the flat resource (no category keys)
    assert!(groups[0]["stats"][0]["weapon"].get("category").is_none());
    assert_eq!(groups[0]["stats"][0]["weapon"]["identifier"], "T4_SWORD");
    assert_eq!(groups[0]["stats"][0]["quality"], "Normal");
    assert_eq!(groups[0]["stats"][0]["stats"][0]["value"], "700");
}

#[tokio::test]
async fn craftings_group_by_enchantment() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    let (status, value) = get(&mut app, "/api/v3/consumable-craftings/consumable/1").await;
    assert_eq!(status, StatusCode::OK);
    let groups = value["data"].as_array().unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0]["enchantment"], 0);
    assert_eq!(groups[0]["crafting"]["per_craft"], 1);
    assert_eq!(groups[0]["crafting"]["requirements"][0]["identifier"], "T3_MEAT");
    assert_eq!(groups[0]["crafting"]["consumable"]["identifier"], "T4_OMELLETTE");
    assert_eq!(groups[0]["icon"], "https://render.albiononline.com/v1/item/T4_OMELLETTE@0.png?quality=0&size=217&locale=en");
    assert_eq!(groups[1]["crafting"]["per_craft"], 2);
}

#[tokio::test]
async fn consumable_stats_match_shape() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    let (status, value) = get(&mut app, "/api/v2/consumable-stats/consumable/1").await;
    assert_eq!(status, StatusCode::OK);
    let groups = value["data"].as_array().unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0]["stats"][0]["consumable"]["info"], "Restores health.");
    assert_eq!(groups[1]["enchantment"], 1);
}

#[tokio::test]
async fn search_behaves_like_the_laravel_queries() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let app = build_app(state.clone());

    // /api/weaponry/search (v1): 3 per type, weapons -> armors -> accessories,
    // missing search matches everything. Public — no key header.
    let plain_req = |uri: &str| Request::builder().uri(uri).body(Body::empty()).unwrap();

    let response = app.clone().oneshot(plain_req("/api/weaponry/search")).await.unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let types: Vec<&str> = value["data"].as_array().unwrap().iter().map(|r| r["type"].as_str().unwrap()).collect();
    assert_eq!(types.len(), 5); // 3 weapons + 1 armor + 1 accessory (limits apply per type)
    // search for "sword" hits weapons only
    let response = app
        .clone()
        .oneshot(plain_req("/api/weaponry/search?search=sword"))
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let names: Vec<&str> = value["data"].as_array().unwrap().iter().map(|r| r["name"].as_str().unwrap()).collect();
    assert!(names.iter().all(|n| n.contains("Sword")));
    assert_eq!(names.len(), 3);
    // case-insensitive
    let response = app
        .clone()
        .oneshot(plain_req("/api/weaponry/search?search=SWORD"))
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["data"].as_array().unwrap().len(), 3);

    // /api/weaponryV2/search: all four types, no per-type cap, info null for
    // non-consumables and set for consumables.
    let response = app
        .clone()
        .oneshot(plain_req("/api/weaponryV2/search?search=omelette"))
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let rows = value["data"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["type"], "consumable");
    assert_eq!(rows[0]["info"], "Restores health.");
    assert_eq!(rows[0]["type_id"], 1);
}

#[tokio::test]
async fn version_check_and_forty_fours() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/weaponry/version-check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["version"], "1.0.7");
    assert_eq!(value["force"], false);
    assert_eq!(value["description"], "[+] Add Food & Potions\n[+] Add Crafting Process");

    // unknown route -> JSON 404
    let (status, body) = get(&mut app, "/api/v3/unknown").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["message"], "Not Found");

    // trailing slash still matches
    let (status, _) = get(&mut app, "/api/v1/weapons/").await;
    assert_eq!(status, StatusCode::OK);
    let response = get_raw(&mut app, "/terms-of-service/").await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn cors_preflight_is_answered_for_api_routes() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let app = build_app(state.clone());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/weaponry/search")
                .header("origin", "http://example.com")
                .header("access-control-request-method", "GET")
                .header("access-control-request-headers", "Authorization")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let allow_origin = response.headers().get("access-control-allow-origin").unwrap();
    assert_eq!(allow_origin, "http://example.com");
}

#[tokio::test]
async fn bug_report_persists_and_validates() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    // Valid: persisted to the JSONL file, success response.
    let (status, value) = post_json(
        &mut app,
        "/api/weaponry/bug-report",
        json!({"category": "UI", "description": "crash", "debug": [1]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["message"], "success");

    let written = std::fs::read_to_string(dir.join("bug-reports.jsonl")).unwrap();
    let record: Value = serde_json::from_str(written.trim_end()).unwrap();
    assert_eq!(record["category"], "UI");
    assert_eq!(record["description"], "crash");
    assert_eq!(record["debug"], json!([1]));
    assert!(record["timestamp"].is_u64());

    // Invalid: Laravel-style 422 with summarized message and per-field errors.
    let (status, value) = post_json(&mut app, "/api/weaponry/bug-report", json!({})).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(value["message"], "The category field is required. (and 2 more errors)");
    assert_eq!(value["errors"]["category"][0], "The category field is required.");
    assert_eq!(value["errors"]["description"][0], "The description field is required.");
    assert_eq!(value["errors"]["debug"][0], "The debug field is required.");

    // debug must be an array
    let (status, value) = post_json(
        &mut app,
        "/api/weaponry/bug-report",
        json!({"category": "UI", "description": "x", "debug": {"a": 1}}),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(value["message"], "The debug must be an array.");
    assert_eq!(value["errors"]["debug"][0], "The debug must be an array.");
}

#[tokio::test]
async fn rate_limit_returns_429() {
    let (dir, _guard) = temp_data_dir();
    let _ = fixture_state(&dir); // writes the fixture tables
    let state = Arc::new(
        AppState::load(
            &dir,
            AppConfig {
                rate_limit_per_minute: 2,
                ..AppConfig::default()
            },
        )
        .unwrap(),
    );
    let mut app = build_app(state);

    let (status, _) = get(&mut app, "/api/v1/weapons").await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = get(&mut app, "/api/v1/armors").await;
    assert_eq!(status, StatusCode::OK);
    // Third request in the same window: over the limit.
    let response = get_raw(&mut app, "/api/v1/accessories").await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(response.headers().contains_key("retry-after"));
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["message"], "Too Many Attempts.");
}

#[tokio::test]
async fn web_pages_serve() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    for (path, title) in [
        (
            "/",
            "<title>OpenAlbion — free Albion Online data API</title>",
        ),
        ("/docs", "<title>Introduction | OpenAlbion Docs</title>"),
        (
            "/terms-of-service",
            "<title>Terms of Service | OpenAlbion</title>",
        ),
        (
            "/privacy-policy",
            "<title>Privacy Policy | OpenAlbion</title>",
        ),
    ] {
        let (status, html) = get_text(&mut app, path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(html.contains(title), "{path} should contain {title}");
        assert!(!html.contains("<img src=\"http"), "{path} should not hotlink images");
        assert!(html.contains("src=\"/logo.png\""), "{path} should use the local logo");
        assert!(!html.contains("weaponryV2"), "{path} must not advertise the app-only V2 endpoint");
        for forbidden in [
            "fonts.bunny",
            "fonts.googleapis",
            "cloudinary",
            "gstatic",
            "algolia",
        ] {
            assert!(
                !html.contains(forbidden),
                "{path} must not reference {forbidden}"
            );
        }
    }

    // The home page is data-driven: fixture counts are injected, no tokens remain.
    let (_, home) = get_text(&mut app, "/").await;
    assert!(home.contains("<dt>3</dt>"), "weapon count is injected");
    assert!(home.contains("<dt>2</dt>"), "recipe count is injected");
    assert!(!home.contains("{{"), "no template tokens should remain");
    assert!(home.contains("/docs/weapons"));
    assert!(home.contains("data-playground"));

    // /download redirects to the Play Store with 302
    let response = get_raw(&mut app, "/download").await;
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "https://play.google.com/store/apps/details?id=com.openalbion.weaponry"
    );

    // robots, favicons, CSS, JS and sitemap
    for path in [
        "/robots.txt",
        "/favicon.ico",
        "/logo.png",
        "/app.css",
        "/site.js",
        "/sitemap.xml",
    ] {
        let response = get_raw(&mut app, path).await;
        assert_eq!(response.status(), StatusCode::OK, "{path}");
    }

    let (_, robots) = get_text(&mut app, "/robots.txt").await;
    assert!(robots.contains("Sitemap: https://api.openalbion.com/sitemap.xml"));

    let (_, sitemap) = get_text(&mut app, "/sitemap.xml").await;
    assert!(sitemap.contains("<loc>https://api.openalbion.com/docs/weapons</loc>"));
    assert!(sitemap.contains("<loc>https://api.openalbion.com/terms-of-service</loc>"));

    let (_, css) = get_text(&mut app, "/app.css").await;
    assert!(!css.contains("@import"));
    assert!(!css.contains("fonts.bunny"));
    assert!(!css.contains("http://"));
    assert!(!css.contains("https://"));
}

#[tokio::test]
async fn docs_pages_serve() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    for doc in openalbion_api::web::docs::DOCS {
        let path = if doc.slug == "introduction" {
            "/docs".to_string()
        } else {
            format!("/docs/{}", doc.slug)
        };
        let (status, html) = get_text(&mut app, &path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        let expected_title =
            openalbion_api::web::markdown::escape_html(&format!("{} | OpenAlbion Docs", doc.title));
        assert!(
            html.contains(&expected_title),
            "{path} should carry the docs title"
        );
        if let Some(endpoint) = doc.endpoint {
            assert!(html.contains(endpoint), "{path} should show {endpoint}");
        }
        assert!(
            html.contains("docs-sidebar"),
            "{path} should render the sidebar"
        );
        assert!(!html.contains("<img src=\"http"), "{path} should not hotlink images");
        assert!(html.contains("src=\"/logo.png\""), "{path} should use the local logo");
        assert!(!html.contains("weaponryV2"), "{path} must not advertise the app-only V2 endpoint");
    }
}

#[tokio::test]
async fn docs_and_legacy_urls_redirect() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    for (from, to) in openalbion_api::web::docs::LEGACY_REDIRECTS {
        let response = get_raw(&mut app, from).await;
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT, "{from}");
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), *to);
    }

    for from in ["/docs/index", "/docs/introduction", "/introduction"] {
        let response = get_raw(&mut app, from).await;
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT, "{from}");
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/docs");
    }
}

#[tokio::test]
async fn unknown_pages_are_themed_404s() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    for path in ["/no-such-page", "/docs/no-such-page"] {
        let (status, html) = get_text(&mut app, path).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
        assert!(html.contains("This page wandered off."), "{path}");
        assert!(html.contains("<link rel=\"stylesheet\" href=\"/app.css\">"));
    }

    // API fallbacks stay JSON.
    let (status, value) = get(&mut app, "/api/nope").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(value["message"], "Not Found");
}

#[tokio::test]
async fn playground_presets_answer() {
    let (dir, _guard) = temp_data_dir();
    let state = fixture_state(&dir);
    let mut app = build_app(state.clone());

    for path in [
        "/api/v3/weapons?tier=8",
        "/api/v3/categories?type=weapon",
        "/api/v3/spells/weapon/1",
        "/api/v3/weapon-stats/weapon/1",
        "/api/v3/consumable-craftings/consumable/1",
        "/api/v3/armors?tier=4",
    ] {
        let (status, value) = get(&mut app, path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(
            value.get("data").is_some(),
            "{path} should return a data key"
        );
    }
}

#[tokio::test]
async fn aod_proxy_forwards_and_caches() {
    // Mock upstream that records the query it received.
    use std::sync::atomic::{AtomicUsize, Ordering};
    let hits = Arc::new(AtomicUsize::new(0));

    let mock = axum::Router::new().route(
        "/prices/{region}/{item}",
        axum::routing::get({
            let hits = hits.clone();
            move |axum::extract::RawQuery(q): axum::extract::RawQuery,
                  axum::extract::Path((region, item)): axum::extract::Path<(String, String)>| {
                let hits = hits.clone();
                async move {
                    hits.fetch_add(1, Ordering::SeqCst);
                    let q = q.unwrap_or_default();
                    axum::Json(json!({
                        "item_id": item,
                        "region": region,
                        "query": q,
                    }))
                }
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mock).await.unwrap();
    });

    let (dir, _guard) = temp_data_dir();
    let _ = fixture_state(&dir); // write the fixture tables
    let state = {
        let config = AppConfig {
            aod_cache_ttl: Duration::from_secs(60),
            aod_url_template: format!("http://{addr}/prices/{{region}}/{{item}}.json"),
            ..AppConfig::default()
        };
        Arc::new(AppState::load(&dir, config).unwrap())
    };
    let app = build_app(state);

    let aod = |query: &str| {
        let uri = format!("/api/weaponry/aod/east/item/T4_BAG/price?{query}");
        Request::builder().uri(uri).body(Body::empty()).unwrap()
    };

    let response = app.clone().oneshot(aod("locations=Caerleon")).await.unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["region"], "east");
    // The upstream really receives the `T4_BAG.json` segment, exactly like
    // the URL Laravel builds.
    assert_eq!(value["item_id"], "T4_BAG.json");
    assert_eq!(value["query"], "locations=Caerleon");

    // Same query again: served from cache (upstream hit count stays 1).
    let response = app.clone().oneshot(aod("locations=Caerleon")).await.unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["query"], "locations=Caerleon");
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    // Different query: upstream called again.
    let response = app.clone().oneshot(aod("locations=Bridgewatch")).await.unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["query"], "locations=Bridgewatch");
    assert_eq!(hits.load(Ordering::SeqCst), 2);
}
