//! `/api/weaponry` and `/api/weaponryV2` specific endpoints.

use std::sync::Arc;

use axum::extract::{Path, RawQuery, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::dto::*;
use crate::state::{AodCacheKey, AppState};

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub search: Option<String>,
}

/// v1 search: LIKE-style match, 3 per type, weapons -> armors -> accessories.
/// A missing `search` param behaves like an empty needle (matches everything).
pub fn search_v1(state: &AppState, q: &SearchQuery) -> Json<DataList<SearchItemDto>> {
    let needle = q.search.clone().unwrap_or_default().to_lowercase();
    let contains = |name: &str| name.to_lowercase().contains(&needle);

    let mut data: Vec<SearchItemDto> = Vec::new();
    data.extend(
        state
            .weapons
            .iter()
            .filter(|w| contains(&w.name))
            .take(3)
            .map(SearchItemDto::weapon),
    );
    data.extend(
        state
            .armors
            .iter()
            .filter(|a| contains(&a.name))
            .take(3)
            .map(SearchItemDto::armor),
    );
    data.extend(
        state
            .accessories
            .iter()
            .filter(|a| contains(&a.name))
            .take(3)
            .map(SearchItemDto::accessory),
    );

    Json(DataList::new(data))
}

/// V2 search: same substring match over all four item types, without a
/// per-type limit (replaces the Algolia/Scout aggregator; ordering is
/// weapons -> armors -> accessories -> consumables instead of relevance).
pub fn search_v2(state: &AppState, q: &SearchQuery) -> Json<DataList<SearchItemDto>> {
    let needle = q.search.clone().unwrap_or_default().to_lowercase();
    let contains = |name: &str| name.to_lowercase().contains(&needle);

    let mut data: Vec<SearchItemDto> = Vec::new();
    data.extend(state.weapons.iter().filter(|w| contains(&w.name)).map(SearchItemDto::weapon));
    data.extend(state.armors.iter().filter(|a| contains(&a.name)).map(SearchItemDto::armor));
    data.extend(
        state
            .accessories
            .iter()
            .filter(|a| contains(&a.name))
            .map(SearchItemDto::accessory),
    );
    data.extend(
        state
            .consumables
            .iter()
            .filter(|c| contains(&c.name))
            .map(SearchItemDto::consumable),
    );

    Json(DataList::new(data))
}

// ---------------------------------------------------------------------------
// Version check
// ---------------------------------------------------------------------------

pub async fn version_check() -> Json<VersionCheckDto> {
    Json(VersionCheckDto::latest())
}

// ---------------------------------------------------------------------------
// Bug report
// ---------------------------------------------------------------------------

/// Parses and validates the request body, mirroring `BugReportRequest`:
/// `category` (required, max 50), `description` (required, max 250),
/// `debug` (required, array).
enum BugReportBody {
    Valid {
        category: String,
        description: String,
        debug: Value,
    },
    Invalid(ValidationErrorDto),
}

fn validate_bug_report(body: &Value) -> BugReportBody {
    let category = body.get("category");
    let description = body.get("description");
    let debug = body.get("debug");

    let mut errors = ValidationErrorsDto {
        category: None,
        description: None,
        debug: None,
    };

    if let Some(c) = category {
        match c.as_str() {
            Some(s) if !s.is_empty() => {
                if s.chars().count() > 50 {
                    errors.category =
                        Some(vec!["The category may not be greater than 50 characters.".into()]);
                }
            }
            _ => errors.category = Some(vec!["The category field is required.".into()]),
        }
    } else {
        errors.category = Some(vec!["The category field is required.".into()]);
    }

    if let Some(d) = description {
        match d.as_str() {
            Some(s) if !s.is_empty() => {
                if s.chars().count() > 250 {
                    errors.description =
                        Some(vec!["The description may not be greater than 250 characters.".into()]);
                }
            }
            _ => errors.description = Some(vec!["The description field is required.".into()]),
        }
    } else {
        errors.description = Some(vec!["The description field is required.".into()]);
    }

    match debug {
        None | Some(Value::Null) => errors.debug = Some(vec!["The debug field is required.".into()]),
        Some(Value::Array(_)) => {}
        Some(_) => errors.debug = Some(vec!["The debug must be an array.".into()]),
    }

    if errors.category.is_some() || errors.description.is_some() || errors.debug.is_some() {
        return BugReportBody::Invalid(ValidationErrorDto::summarize(errors));
    }

    BugReportBody::Valid {
        category: category.and_then(Value::as_str).unwrap_or_default().to_string(),
        description: description.and_then(Value::as_str).unwrap_or_default().to_string(),
        debug: debug.cloned().unwrap_or(Value::Null),
    }
}

pub async fn bug_report(
    State(state): State<Arc<AppState>>,
    body: Result<Json<Value>, axum::extract::rejection::JsonRejection>,
) -> Response {
    // An unparseable JSON body is validated like an empty one (Laravel
    // silently fails to decode it and every field comes back as required).
    let body = match body {
        Ok(Json(value)) => value,
        Err(_) => json!({}),
    };

    match validate_bug_report(&body) {
        BugReportBody::Invalid(dto) => (StatusCode::UNPROCESSABLE_ENTITY, Json(dto)).into_response(),
        BugReportBody::Valid { category, description, debug } => {
            // Persist (JSONL file on the server, Workers log on Cloudflare).
            let record = json!({
                "timestamp": crate::platform::now_unix(),
                "category": category,
                "description": description,
                "debug": debug,
            });
            crate::platform::store_bug_report(&state, &record);

            // Forward to Slack like `Spatie\SlackAlerts` when configured.
            if let Some(webhook) = state.slack_webhook_url.clone() {
                let debug_pretty = serde_json::to_string_pretty(&debug).unwrap_or_else(|_| "{}".into());
                let message = format!("*{category}*\n{description}\n```{debug_pretty}```");
                crate::platform::post_json(&state, webhook, json!({ "text": message })).await;
            }

            Json(MessageDto::new("success")).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// AOD price proxy
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AodPath {
    pub region: String,
    pub item_id: String,
}

fn parse_query_pairs(raw: &Option<String>) -> Vec<(String, String)> {
    match raw {
        Some(q) => form_urlencoded_parse(q),
        None => Vec::new(),
    }
}

/// Minimal `application/x-www-form-urlencoded` pair parser (no extra crate).
fn form_url_encoded_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn form_urlencoded_parse(q: &str) -> Vec<(String, String)> {
    q.split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((k, v)) => (form_url_encoded_decode(k), form_url_encoded_decode(v)),
            None => (form_url_encoded_decode(pair), String::new()),
        })
        .collect()
}

pub async fn aod_price(
    State(state): State<Arc<AppState>>,
    Path(AodPath { region, item_id }): Path<AodPath>,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let pairs = parse_query_pairs(&raw_query);

    // Cache key: region + item + sorted query minus `api_token` — mirrors
    // Laravel's `generateCacheKey`.
    let mut key_query: Vec<(String, String)> = pairs
        .iter()
        .filter(|(k, _)| k != "api_token")
        .cloned()
        .collect();
    key_query.sort();

    let cache_key = AodCacheKey {
        region: region.clone(),
        item_id: item_id.clone(),
        query: key_query,
    };
    if let Some(cached) = state.aod_cache.get(&cache_key) {
        return Json(cached).into_response();
    }

    let url = state
        .aod_url_template
        .replace("{region}", &region)
        .replace("{item}", &item_id);

    let value = match crate::platform::http_get_text(&state, &url, &pairs).await {
        // Laravel does `Http::get(...)->json()`: an unparseable body
        // becomes `null`, and the upstream status is *not* propagated.
        Ok(text) => serde_json::from_str::<Value>(&text).unwrap_or(Value::Null),
        // Laravel throws (500 HTML) on transport errors; 502 JSON is the
        // documented deviation.
        Err(e) => {
            tracing::warn!(error = %e, "aod upstream fetch failed");
            return (
                StatusCode::BAD_GATEWAY,
                Json(MessageDto::new("Failed to fetch data from Albion Online Data.")),
            )
                .into_response();
        }
    };

    state.aod_cache.put(cache_key, value.clone());
    Json(value).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_url_encoded_pairs() {
        let pairs = form_urlencoded_parse("q=Breakfast%20Pie&locations=Caerleon&n=1");
        assert_eq!(
            pairs,
            vec![
                ("q".to_string(), "Breakfast Pie".to_string()),
                ("locations".to_string(), "Caerleon".to_string()),
                ("n".to_string(), "1".to_string()),
            ]
        );
    }

    #[test]
    fn decodes_multi_byte_utf8() {
        let pairs = form_urlencoded_parse("s=%C3%A9");
        assert_eq!(pairs, vec![("s".to_string(), "é".to_string())]);
    }

    #[test]
    fn bug_report_validation_requires_all_fields() {
        // All missing
        match validate_bug_report(&json!({})) {
            BugReportBody::Invalid(dto) => {
                assert_eq!(dto.message, "The category field is required. (and 2 more errors)");
                assert_eq!(
                    dto.errors.category,
                    Some(vec!["The category field is required.".to_string()])
                );
                assert_eq!(
                    dto.errors.description,
                    Some(vec!["The description field is required.".to_string()])
                );
                assert_eq!(dto.errors.debug, Some(vec!["The debug field is required.".to_string()]));
            }
            _ => panic!("expected invalid"),
        }

        // Debug must be an array
        match validate_bug_report(&json!({"category": "UI", "description": "crash", "debug": {}})) {
            BugReportBody::Invalid(dto) => {
                assert_eq!(dto.message, "The debug must be an array.");
                assert_eq!(dto.errors.debug, Some(vec!["The debug must be an array.".to_string()]));
            }
            _ => panic!("expected invalid"),
        }

        // Too long values
        let long = "x".repeat(51);
        match validate_bug_report(&json!({"category": long, "description": "d", "debug": []})) {
            BugReportBody::Invalid(dto) => {
                assert_eq!(
                    dto.errors.category,
                    Some(vec!["The category may not be greater than 50 characters.".to_string()])
                );
                assert_eq!(dto.message, "The category may not be greater than 50 characters.");
            }
            _ => panic!("expected invalid"),
        }

        // Valid
        match validate_bug_report(&json!({"category": "UI", "description": "crash", "debug": [1, 2]})) {
            BugReportBody::Valid { category, description, debug } => {
                assert_eq!(category, "UI");
                assert_eq!(description, "crash");
                assert_eq!(debug, json!([1, 2]));
            }
            _ => panic!("expected valid"),
        }
    }
}
