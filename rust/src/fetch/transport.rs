//! Page fetching.
//!
//! `wiki.albiononline.com` sits behind Cloudflare: plain HTTP gets a
//! "Just a moment…" challenge and headless Chrome is blocked as well. The
//! practical workflow is therefore:
//!
//! 1. `openalbion-fetch bootstrap` — a real (visible by default) Chrome
//!    solves the challenge once; its cookies + user agent are saved to
//!    `data/.cache/cloudflare.json`.
//! 2. the crawler uses those cookies over plain HTTP (fast, concurrent); on
//!    a challenge it reports clearly that `bootstrap` must be re-run.
//! 3. `--transport browser` runs every request through Chrome instead, for
//!    when the cookie approach is not accepted.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::network::Cookie;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::fetch::report::utc_timestamp;
use crate::fetch::text::wiki_url;

pub const WIKI_BASE: &str = "https://wiki.albiononline.com";

/// Marker present on the Cloudflare interstitial.
const CHALLENGE_MARKER: &str = "Just a moment";

/// Marker present on real wiki pages.
const CONTENT_MARKER: &str = "mw-content-text";

/// A desktop Chrome user agent used when bootstrapping/falling back.
pub const FALLBACK_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/157.0.0.0 Safari/537.36";

#[derive(Debug)]
pub enum FetchError {
    /// Cloudflare interstitial instead of the page.
    Challenge { url: String },
    /// Non-success HTTP status.
    Status { url: String, status: u16 },
    /// Connection error after retries.
    Network { url: String, message: String },
    /// No saved Cloudflare clearance — `bootstrap` was never run.
    NotBootstrapped,
    /// Browser automation failed.
    Browser(String),
    /// Fixture transport: no fixture file for the URL.
    FixtureMissing { url: String, path: String },
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::Challenge { url } => write!(
                f,
                "Cloudflare challenge for {url} — run `openalbion-fetch bootstrap` to refresh the clearance cookie"
            ),
            FetchError::Status { url, status } => write!(f, "HTTP {status} for {url}"),
            FetchError::Network { url, message } => write!(f, "network error for {url}: {message}"),
            FetchError::NotBootstrapped => write!(
                f,
                "no Cloudflare clearance saved — run `openalbion-fetch bootstrap` first"
            ),
            FetchError::Browser(message) => write!(f, "browser error: {message}"),
            FetchError::FixtureMissing { url, path } => {
                write!(f, "no fixture for {url} (expected {path})")
            }
        }
    }
}

impl std::error::Error for FetchError {}

/// Saved Cloudflare clearance (cookies + the UA they are bound to).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareAuth {
    pub user_agent: String,
    /// `Cookie:` header value, e.g. `cf_clearance=…; __cf_bm=…`.
    pub cookie_header: String,
    pub saved_at: String,
}

impl CloudflareAuth {
    pub fn from_cookies(cookies: &[Cookie], user_agent: &str) -> Self {
        let cookie_header = cookies
            .iter()
            .filter(|c| c.domain.contains("albiononline"))
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ");
        Self {
            user_agent: user_agent.to_string(),
            cookie_header,
            saved_at: utc_timestamp(),
        }
    }

    pub fn has_clearance(&self) -> bool {
        self.cookie_header.contains("cf_clearance")
    }
}

pub fn auth_path(data_dir: &Path) -> PathBuf {
    data_dir.join(".cache").join("cloudflare.json")
}

pub fn load_auth(path: &Path) -> io::Result<Option<CloudflareAuth>> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path)?;
    let auth: CloudflareAuth = serde_json::from_str(&text).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid {}: {e}", path.display()),
        )
    })?;
    Ok(Some(auth))
}

pub fn save_auth(path: &Path, auth: &CloudflareAuth) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(auth).map_err(io::Error::other)?,
    )
}

// ---------------------------------------------------------------------------
// Transport
// ---------------------------------------------------------------------------

pub enum Transport {
    Http(HttpFetcher),
    Browser(BrowserFetcher),
    Fixture(FixtureFetcher),
}

pub enum TransportMode {
    Cookie,
    Browser,
    Fixture(PathBuf),
}

impl Transport {
    pub async fn new(
        mode: &TransportMode,
        data_dir: &Path,
        chrome: Option<PathBuf>,
        headless: bool,
        cookie_file: Option<PathBuf>,
    ) -> Result<Self, FetchError> {
        match mode {
            TransportMode::Cookie => {
                let path = cookie_file.unwrap_or_else(|| auth_path(data_dir));
                let auth = load_auth(&path).map_err(|e| FetchError::Network {
                    url: path.display().to_string(),
                    message: e.to_string(),
                })?;
                Ok(Transport::Http(HttpFetcher::new(auth)))
            }
            TransportMode::Browser => {
                let browser = BrowserFetcher::launch(chrome, headless).await?;
                Ok(Transport::Browser(browser))
            }
            TransportMode::Fixture(dir) => Ok(Transport::Fixture(FixtureFetcher::new(dir.clone()))),
        }
    }

    pub async fn get_html(&self, url: &str) -> Result<String, FetchError> {
        match self {
            Transport::Http(fetcher) => fetcher.get_html(url).await,
            Transport::Browser(fetcher) => fetcher.get_html(url).await,
            Transport::Fixture(fetcher) => fetcher.get_html(url),
        }
    }
}

// ---------------------------------------------------------------------------
// Cookie HTTP transport
// ---------------------------------------------------------------------------

pub struct HttpFetcher {
    client: reqwest::Client,
    auth: Option<CloudflareAuth>,
    cache: Mutex<HashMap<String, String>>,
}

impl HttpFetcher {
    pub fn new(auth: Option<CloudflareAuth>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("http client");
        Self {
            client,
            auth,
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn cache_get(&self, url: &str) -> Option<String> {
        self.cache.lock().ok()?.get(url).cloned()
    }

    fn cache_put(&self, url: &str, html: &str) {
        if let Ok(mut guard) = self.cache.lock() {
            guard.insert(url.to_string(), html.to_string());
        }
    }

    pub async fn get_html(&self, url: &str) -> Result<String, FetchError> {
        if let Some(cached) = self.cache_get(url) {
            return Ok(cached);
        }
        let auth = self.auth.clone().ok_or(FetchError::NotBootstrapped)?;

        let mut last_error = None;
        for attempt in 0..3u32 {
            let response = self
                .client
                .get(url)
                .header(reqwest::header::USER_AGENT, &auth.user_agent)
                .header(
                    reqwest::header::ACCEPT,
                    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                )
                .header(reqwest::header::ACCEPT_LANGUAGE, "en-US,en;q=0.9")
                .header(reqwest::header::COOKIE, &auth.cookie_header)
                .header(reqwest::header::REFERER, WIKI_BASE)
                .send()
                .await;

            match response {
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    if status == reqwest::StatusCode::FORBIDDEN
                        || status == reqwest::StatusCode::SERVICE_UNAVAILABLE
                        || (body.contains(CHALLENGE_MARKER) && !body.contains(CONTENT_MARKER))
                    {
                        return Err(FetchError::Challenge {
                            url: url.to_string(),
                        });
                    }
                    if !status.is_success() {
                        return Err(FetchError::Status {
                            url: url.to_string(),
                            status: status.as_u16(),
                        });
                    }
                    self.cache_put(url, &body);
                    return Ok(body);
                }
                Err(error) => {
                    last_error = Some(error.to_string());
                    tokio::time::sleep(Duration::from_millis(500 * (attempt as u64 + 1))).await;
                }
            }
        }
        Err(FetchError::Network {
            url: url.to_string(),
            message: last_error.unwrap_or_else(|| "unknown error".to_string()),
        })
    }
}

// ---------------------------------------------------------------------------
// Browser transport (visible by default — Cloudflare blocks headless)
// ---------------------------------------------------------------------------

pub struct BrowserFetcher {
    browser: Browser,
    _handler: tokio::task::JoinHandle<()>,
    cache: Mutex<HashMap<String, String>>,
}

impl BrowserFetcher {
    pub async fn launch(chrome: Option<PathBuf>, headless: bool) -> Result<Self, FetchError> {
        let mut builder = BrowserConfig::builder().no_sandbox();
        if let Some(path) = chrome {
            builder = builder.chrome_executable(path);
        }
        if !headless {
            builder = builder.with_head();
        }
        let config = builder.build().map_err(FetchError::Browser)?;
        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| FetchError::Browser(e.to_string()))?;
        let handle = tokio::spawn(async move { while handler.next().await.is_some() {} });
        Ok(Self {
            browser,
            _handler: handle,
            cache: Mutex::new(HashMap::new()),
        })
    }

    fn cache_get(&self, url: &str) -> Option<String> {
        self.cache.lock().ok()?.get(url).cloned()
    }

    fn cache_put(&self, url: &str, html: &str) {
        if let Ok(mut guard) = self.cache.lock() {
            guard.insert(url.to_string(), html.to_string());
        }
    }

    /// Opens the URL in a tab and waits for real wiki content (solving the
    /// Cloudflare interstitial counts as "waiting").
    async fn open_page(&self, url: &str) -> Result<chromiumoxide::Page, FetchError> {
        let page = self
            .browser
            .new_page(url)
            .await
            .map_err(|e| FetchError::Browser(e.to_string()))?;

        for _ in 0..90 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let html = match page.content().await {
                Ok(html) => html,
                Err(_) => continue,
            };
            if html.contains(CONTENT_MARKER) && !html.contains(CHALLENGE_MARKER) {
                return Ok(page);
            }
        }
        let _ = page.close().await;
        Err(FetchError::Challenge {
            url: url.to_string(),
        })
    }

    pub async fn get_html(&self, url: &str) -> Result<String, FetchError> {
        if let Some(cached) = self.cache_get(url) {
            return Ok(cached);
        }
        let page = self.open_page(url).await?;
        let html = page
            .content()
            .await
            .map_err(|e| FetchError::Browser(e.to_string()))?;
        let _ = page.close().await;
        self.cache_put(url, &html);
        Ok(html)
    }

    /// Solves the Cloudflare challenge once and captures cookies + UA.
    pub async fn bootstrap(&self, url: &str) -> Result<CloudflareAuth, FetchError> {
        let page = self.open_page(url).await?;
        let user_agent = page
            .evaluate("navigator.userAgent")
            .await
            .ok()
            .and_then(|result| result.value().and_then(|v| v.as_str()).map(str::to_string))
            .unwrap_or_else(|| FALLBACK_USER_AGENT.to_string());
        let cookies = page
            .get_cookies()
            .await
            .map_err(|e| FetchError::Browser(e.to_string()))?;
        let _ = page.close().await;
        Ok(CloudflareAuth::from_cookies(&cookies, &user_agent))
    }
}

// ---------------------------------------------------------------------------
// Fixture transport (tests / offline parser work)
// ---------------------------------------------------------------------------

/// File name used for `--dump-html` output and fixture lookup.
pub fn fixture_name(url: &str) -> String {
    let path = url.strip_prefix(WIKI_BASE).unwrap_or(url);
    let sanitized = path.trim_start_matches('/').replace(['/', '%'], "_");
    format!("{sanitized}.html")
}

pub struct FixtureFetcher {
    dir: PathBuf,
}

impl FixtureFetcher {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn fixture_path(&self, url: &str) -> PathBuf {
        self.dir.join(fixture_name(url))
    }

    pub fn get_html(&self, url: &str) -> Result<String, FetchError> {
        let path = self.fixture_path(url);
        fs::read_to_string(&path).map_err(|_| FetchError::FixtureMissing {
            url: url.to_string(),
            path: path.display().to_string(),
        })
    }
}

/// Convenience: full wiki URL for a page path.
pub fn wiki(url: &str) -> String {
    if url.starts_with("http") {
        url.to_string()
    } else {
        wiki_url(WIKI_BASE, url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_paths_are_filesystem_friendly() {
        let fetcher = FixtureFetcher::new(PathBuf::from("/tmp/fixtures"));
        let path = fetcher.fixture_path("https://wiki.albiononline.com/wiki/Adept%27s_Sword");
        assert_eq!(
            path,
            PathBuf::from("/tmp/fixtures/wiki_Adept_27s_Sword.html")
        );
    }

    #[test]
    fn fixture_transport_reads_files() {
        let dir = std::env::temp_dir().join(format!(
            "openalbion-fixture-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("wiki_Axe.html"),
            "<div id=\"mw-content-text\">x</div>",
        )
        .unwrap();
        let fetcher = FixtureFetcher::new(dir.clone());
        let html = fetcher
            .get_html("https://wiki.albiononline.com/wiki/Axe")
            .unwrap();
        assert!(html.contains("mw-content-text"));
        assert!(matches!(
            fetcher.get_html("https://wiki.albiononline.com/wiki/Missing"),
            Err(FetchError::FixtureMissing { .. })
        ));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cloudflare_auth_extracts_cookie_header() {
        // Deserialise from CDP JSON so the generated struct's exact field set
        // is not duplicated in the test.
        let cookies: Vec<Cookie> = serde_json::from_value(serde_json::json!([{
            "name": "cf_clearance",
            "value": "abc",
            "domain": ".wiki.albiononline.com",
            "path": "/",
            "expires": 0.0,
            "size": 0,
            "httpOnly": false,
            "secure": true,
            "session": false,
            "priority": "Medium",
            "sourceScheme": "NonSecure",
            "sourcePort": -1
        }]))
        .unwrap();
        let auth = CloudflareAuth::from_cookies(&cookies, "UA");
        assert_eq!(auth.cookie_header, "cf_clearance=abc");
        assert!(auth.has_clearance());
        assert_eq!(auth.user_agent, "UA");
    }

    #[test]
    fn auth_roundtrips_through_a_file() {
        let dir = std::env::temp_dir().join(format!("openalbion-auth-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = auth_path(&dir);
        let auth = CloudflareAuth {
            user_agent: "UA".to_string(),
            cookie_header: "cf_clearance=abc".to_string(),
            saved_at: "now".to_string(),
        };
        save_auth(&path, &auth).unwrap();
        let loaded = load_auth(&path).unwrap().unwrap();
        assert_eq!(loaded.cookie_header, auth.cookie_header);
        let _ = fs::remove_dir_all(dir);
    }
}
