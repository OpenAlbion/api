//! Albion Online Data item dump (`ao-bin-dumps/formatted/items.json`).
//!
//! The identifier fixups and the crafting requirements resolve English item
//! names to the game's `UniqueName` through this dump, exactly like
//! `AlbionOnlineDataService` did.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime};

use serde::Deserialize;

pub const AOD_ITEMS_URL: &str =
    "https://github.com/ao-data/ao-bin-dumps/raw/master/formatted/items.json";

/// Reuse a cached dump for a day.
const CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Deserialize)]
struct AodItem {
    #[serde(rename = "UniqueName")]
    unique_name: Option<String>,
    #[serde(rename = "LocalizedNames")]
    localized_names: Option<HashMap<String, String>>,
}

#[derive(Debug, Default)]
pub struct AodItems {
    /// lower-cased English name → UniqueName (first entry wins).
    by_name: HashMap<String, String>,
}

impl AodItems {
    pub fn from_json(text: &str) -> io::Result<Self> {
        let items: Vec<AodItem> = serde_json::from_str(text).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid AOD items JSON: {e}"),
            )
        })?;
        let mut by_name = HashMap::with_capacity(items.len());
        for item in items {
            let (Some(unique_name), Some(localized)) = (item.unique_name, item.localized_names)
            else {
                continue;
            };
            if let Some(name) = localized.get("EN-US") {
                by_name.entry(name.to_lowercase()).or_insert(unique_name);
            }
        }
        Ok(Self { by_name })
    }

    /// Case-insensitive lookup by localized English name.
    pub fn unique_name(&self, name: &str) -> Option<&str> {
        self.by_name.get(&name.to_lowercase()).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}

fn cache_is_fresh(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    SystemTime::now()
        .duration_since(modified)
        .map(|age| age < CACHE_TTL)
        .unwrap_or(false)
}

/// Loads the cached dump, downloading it when missing, stale or `refresh`.
pub async fn load_or_download(cache_path: &Path, refresh: bool) -> io::Result<AodItems> {
    if !refresh && cache_is_fresh(cache_path) {
        let text = fs::read_to_string(cache_path)?;
        return AodItems::from_json(&text);
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(io::Error::other)?;
    let response = client
        .get(AOD_ITEMS_URL)
        .header(reqwest::header::USER_AGENT, "openalbion-fetch")
        .send()
        .await
        .map_err(|e| io::Error::other(format!("downloading AOD items: {e}")))?;
    if !response.status().is_success() {
        return Err(io::Error::other(format!(
            "downloading AOD items: HTTP {}",
            response.status()
        )));
    }
    let text = response.text().await.map_err(io::Error::other)?;
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(cache_path, &text)?;
    AodItems::from_json(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_case_insensitive_lookup() {
        let json = r#"[
            {"UniqueName": "T4_MILK", "LocalizedNames": {"EN-US": "Goat's Milk"}},
            {"UniqueName": "T3_MEAT", "LocalizedNames": {"EN-US": "Raw Chicken"}},
            {"UniqueName": "T4_MILK@1", "LocalizedNames": {"EN-US": "Goat's Milk"}},
            {"UniqueName": "T4_NO_NAME", "LocalizedNames": {}}
        ]"#;
        let items = AodItems::from_json(json).unwrap();
        // First entry wins, case-insensitively.
        assert_eq!(items.unique_name("Goat's Milk"), Some("T4_MILK"));
        assert_eq!(items.unique_name("goat's milk"), Some("T4_MILK"));
        assert_eq!(items.unique_name("Raw Chicken"), Some("T3_MEAT"));
        assert_eq!(items.unique_name("Missing"), None);
        assert_eq!(items.len(), 2);
    }
}
