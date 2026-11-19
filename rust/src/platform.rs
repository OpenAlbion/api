//! The few things that differ between the native server and the
//! Cloudflare Workers build: the clock, outgoing HTTP and the bug-report
//! sink. Everything else in the crate is platform-agnostic.

use serde_json::Value;

use crate::state::AppState;

/// Milliseconds since the Unix epoch.
///
/// `std::time::{Instant, SystemTime}::now()` panic on
/// `wasm32-unknown-unknown`, so the Workers build asks JavaScript instead.
pub fn now_millis() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys_now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

#[cfg(all(target_arch = "wasm32", feature = "worker"))]
fn js_sys_now() -> u64 {
    worker::js_sys::Date::now() as u64
}

#[cfg(all(target_arch = "wasm32", not(feature = "worker")))]
fn js_sys_now() -> u64 {
    0
}

pub fn now_unix() -> i64 {
    (now_millis() / 1000) as i64
}

/// GET `url` with `query` appended and return the response body.
/// `Err` is a transport failure (the caller answers 502).
pub async fn http_get_text(state: &AppState, url: &str, query: &[(String, String)]) -> Result<String, String> {
    #[cfg(feature = "server")]
    {
        let resp = state
            .http_client
            .get(url)
            .query(query)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        // A body that fails to read is treated like an unparseable body.
        Ok(resp.text().await.unwrap_or_default())
    }
    #[cfg(all(feature = "worker", not(feature = "server")))]
    {
        let _ = state;
        let mut url = worker::Url::parse(url).map_err(|e| e.to_string())?;
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }
        // Workers futures are !Send; axum handlers must be Send. Workers are
        // single-threaded, so SendFuture is sound here.
        worker::send::SendFuture::new(async move {
            let mut resp = worker::Fetch::Url(url).send().await.map_err(|e| e.to_string())?;
            Ok(resp.text().await.unwrap_or_default())
        })
        .await
    }
    #[cfg(not(any(feature = "server", feature = "worker")))]
    {
        let _ = (state, url, query);
        Err("no HTTP client in this build".to_string())
    }
}

/// POST `body` as JSON to `url`. On the server this runs in the background;
/// on Workers it is awaited (background tasks are cut off once the response
/// is sent unless registered with `waitUntil`).
pub async fn post_json(state: &AppState, url: String, body: Value) {
    #[cfg(feature = "server")]
    {
        let client = state.http_client.clone();
        tokio::spawn(async move {
            if let Err(e) = client.post(&url).json(&body).send().await {
                tracing::warn!(error = %e, "failed to POST {url}");
            }
        });
    }
    #[cfg(all(feature = "worker", not(feature = "server")))]
    {
        let _ = state;
        let result = worker::send::SendFuture::new(async move {
            let headers = worker::Headers::new();
            headers.set("content-type", "application/json")?;
            let mut init = worker::RequestInit::new();
            init.with_method(worker::Method::Post)
                .with_headers(headers)
                .with_body(Some(body.to_string().into()));
            let req = worker::Request::new_with_init(&url, &init)?;
            worker::Fetch::Request(req).send().await.map(|_| ())
        })
        .await;
        if let Err(e) = result {
            worker::console_warn!("failed to POST webhook: {e}");
        }
    }
    #[cfg(not(any(feature = "server", feature = "worker")))]
    {
        let _ = (state, url, body);
    }
}

/// Persists one bug-report record. The server appends to
/// `$DATA_DIR/bug-reports.jsonl`; Workers have no filesystem, so the record
/// goes to the Workers log (visible in the dashboard / `wrangler tail`).
pub fn store_bug_report(state: &AppState, record: &Value) {
    #[cfg(feature = "server")]
    {
        if let Some(path) = &state.bug_reports_path {
            let line = format!("{record}\n");
            if let Err(e) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()))
            {
                tracing::error!(path = %path.display(), error = %e, "failed to append bug report");
            }
        }
    }
    #[cfg(all(feature = "worker", not(feature = "server")))]
    {
        let _ = state;
        worker::console_log!("bug-report {record}");
    }
    #[cfg(not(any(feature = "server", feature = "worker")))]
    {
        let _ = (state, record);
    }
}
