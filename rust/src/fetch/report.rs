//! Backups and the old-vs-fetched diff report.
//!
//! Invariants (see the design notes in `fetch/mod.rs`):
//! - a backup of the previous dataset is taken before writing;
//! - rows missing from a fetch are reported, never deleted;
//! - every run leaves a machine-readable `report.json` next to the backup.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::fetch::store::{TableId, Tables};

/// One table's old-vs-fetched comparison.
#[derive(Debug, Clone)]
pub struct TableDiff {
    pub table: TableId,
    pub old_count: usize,
    /// Distinct keys the fetch actually produced.
    pub fetched_count: usize,
    /// In the fetch but not in the old data.
    pub added: Vec<String>,
    /// In the old data but not in the fetch — kept, warned about.
    pub missing: Vec<String>,
    /// The command only saw part of this table, so `missing` is left empty.
    pub partial: bool,
}

/// The result of one `openalbion-fetch` invocation.
#[derive(Debug, Default)]
pub struct RunReport {
    pub command: String,
    pub created_at: String,
    pub diffs: Vec<TableDiff>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl RunReport {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            created_at: utc_timestamp(),
            ..Self::default()
        }
    }

    /// Builds the per-table diff from the pre-run snapshot and the keys the
    /// fetch produced.
    pub fn compute(
        command: &str,
        old: &Tables,
        fetched: &HashMap<TableId, BTreeSet<String>>,
        partial: &HashSet<TableId>,
    ) -> Self {
        let mut report = Self::new(command);
        let mut tables: Vec<TableId> = fetched.keys().copied().collect();
        tables.sort();
        for table in tables {
            let old_keys = old.keys(table);
            let fetched_keys = fetched.get(&table).cloned().unwrap_or_default();
            let added: Vec<String> = fetched_keys.difference(&old_keys).cloned().collect();
            let is_partial = partial.contains(&table);
            let missing: Vec<String> = if is_partial {
                Vec::new()
            } else {
                old_keys.difference(&fetched_keys).cloned().collect()
            };
            report.diffs.push(TableDiff {
                table,
                old_count: old_keys.len(),
                fetched_count: fetched_keys.len(),
                added,
                missing,
                partial: is_partial,
            });
        }
        report
    }

    pub fn warn(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }

    pub fn has_problems(&self) -> bool {
        !self.warnings.is_empty() || !self.errors.is_empty()
    }

    /// Prints the summary table plus capped added/missing examples.
    pub fn print(&self) {
        if !self.diffs.is_empty() {
            println!();
            println!(
                "{:<22} {:>7} {:>8} {:>6} {:>8}",
                "table", "old", "fetched", "added", "missing"
            );
            for diff in &self.diffs {
                let missing = if diff.partial {
                    "-".to_string()
                } else {
                    diff.missing.len().to_string()
                };
                println!(
                    "{:<22} {:>7} {:>8} {:>6} {:>8}",
                    diff.table.file().trim_end_matches(".json"),
                    diff.old_count,
                    diff.fetched_count,
                    diff.added.len(),
                    missing
                );
            }
            println!();
        }

        for diff in &self.diffs {
            if !diff.added.is_empty() {
                println!(
                    "info: {} new {} row(s): {}",
                    diff.added.len(),
                    diff.table.file().trim_end_matches(".json"),
                    summarize(&diff.added)
                );
            }
            if !diff.missing.is_empty() {
                println!(
                    "warning: {} {} row(s) from the old data were not returned by this fetch \
                     (kept, not deleted): {}",
                    diff.missing.len(),
                    diff.table.file().trim_end_matches(".json"),
                    summarize(&diff.missing)
                );
            }
        }

        for warning in &self.warnings {
            println!("warning: {warning}");
        }
        for error in &self.errors {
            println!("error: {error}");
        }
    }

    /// Writes `report.json` into `dir` (the backup folder, when there is one).
    pub fn write_json(&self, dir: &Path) -> io::Result<PathBuf> {
        fs::create_dir_all(dir)?;
        let value = json!({
            "command": self.command,
            "created_at": self.created_at,
            "tables": self.diffs.iter().map(|d| json!({
                "table": d.table.file().trim_end_matches(".json"),
                "old_count": d.old_count,
                "fetched_count": d.fetched_count,
                "added": d.added,
                "missing": d.missing,
                "partial": d.partial,
            })).collect::<Vec<Value>>(),
            "warnings": self.warnings,
            "errors": self.errors,
        });
        let path = dir.join("report.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&value).map_err(io::Error::other)?,
        )?;
        Ok(path)
    }
}

fn summarize(items: &[String]) -> String {
    const LIMIT: usize = 8;
    if items.len() <= LIMIT {
        items.join(", ")
    } else {
        format!(
            "{} … (+{} more, see report.json)",
            items[..LIMIT].join(", "),
            items.len() - LIMIT
        )
    }
}

/// Copies every existing table file into `data/backups/<UTC timestamp>/`.
/// Returns `None` when there is nothing to back up (first bootstrap).
pub fn backup_dataset(dir: &Path) -> io::Result<Option<PathBuf>> {
    let backup_dir = dir.join("backups").join(utc_timestamp_compact());
    let mut copied = 0usize;
    for table in TableId::ALL {
        let path = dir.join(table.file());
        if !path.exists() {
            continue;
        }
        if copied == 0 {
            fs::create_dir_all(&backup_dir)?;
        }
        fs::copy(&path, backup_dir.join(table.file()))?;
        copied += 1;
    }
    if copied == 0 {
        Ok(None)
    } else {
        Ok(Some(backup_dir))
    }
}

/// `YYYY-MM-DDTHH:MM:SSZ`
pub fn utc_timestamp() -> String {
    let (y, m, d, hh, mm, ss) = utc_parts();
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

/// `YYYYMMDDTHHMMSSZ` (folder names).
pub fn utc_timestamp_compact() -> String {
    let (y, m, d, hh, mm, ss) = utc_parts();
    format!("{y:04}{m:02}{d:02}T{hh:02}{mm:02}{ss:02}Z")
}

fn utc_parts() -> (i64, u32, u32, u32, u32, u32) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rem = (secs % 86_400) as u32;
    let (y, m, d) = civil_from_days(days);
    (y, m, d, rem / 3600, (rem % 3600) / 60, rem % 60)
}

/// Howard Hinnant's `civil_from_days` (proleptic Gregorian).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::store::{CategoryRow, ItemRow};
    use serde_json::json;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "openalbion-report-{}-{}-{}",
            name,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn item(id: i64, name: &str) -> ItemRow {
        ItemRow {
            id,
            category_id: Some(1),
            subcategory_id: Some(2),
            name: name.to_string(),
            identifier: None,
            tier: Some(4.0),
            item_power: None,
            path: None,
        }
    }

    #[test]
    fn diff_reports_added_and_missing_without_deleting() {
        let old = Tables {
            weapons: vec![item(1, "A"), item(2, "B")],
            ..Tables::default()
        };
        let mut fetched = HashMap::new();
        fetched.insert(
            TableId::Weapons,
            ["B".to_string(), "C".to_string()]
                .into_iter()
                .collect::<BTreeSet<_>>(),
        );
        let report = RunReport::compute("weapon", &old, &fetched, &HashSet::new());
        assert_eq!(report.diffs.len(), 1);
        let diff = &report.diffs[0];
        assert_eq!(diff.old_count, 2);
        assert_eq!(diff.fetched_count, 2);
        assert_eq!(diff.added, vec!["C".to_string()]);
        assert_eq!(diff.missing, vec!["A".to_string()]);
        // The old dataset object is untouched — no deletion anywhere.
        assert_eq!(old.weapons.len(), 2);
    }

    #[test]
    fn backup_copies_all_existing_tables() {
        let dir = temp_dir("backup");
        fs::write(dir.join("weapons.json"), "[]").unwrap();
        fs::write(dir.join("categories.json"), "[]").unwrap();
        fs::write(dir.join("notes.txt"), "ignored").unwrap();
        let backup = backup_dataset(&dir).unwrap().expect("backup created");
        assert!(backup.join("weapons.json").exists());
        assert!(backup.join("categories.json").exists());
        assert!(!backup.join("notes.txt").exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn backup_is_none_when_there_is_no_data() {
        let dir = temp_dir("backup-empty");
        assert!(backup_dataset(&dir).unwrap().is_none());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn report_json_contains_counts_and_lists() {
        let dir = temp_dir("reportjson");
        let old = Tables {
            categories: vec![CategoryRow {
                id: 1,
                name: "Old Cat".to_string(),
                parent_id: None,
                path: None,
                item_type: "weapon".to_string(),
                version: 1,
            }],
            ..Tables::default()
        };
        let mut fetched = HashMap::new();
        fetched.insert(
            TableId::Categories,
            ["New Cat".to_string()].into_iter().collect(),
        );
        let report = RunReport::compute("categories", &old, &fetched, &HashSet::new());
        let path = report.write_json(&dir).unwrap();
        let value: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(value["tables"][0]["old_count"], 1);
        assert_eq!(value["tables"][0]["fetched_count"], 1);
        assert_eq!(value["tables"][0]["missing"][0], "Old Cat");
        assert_eq!(value["tables"][0]["added"][0], "New Cat");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn timestamps_are_utc_iso() {
        let ts = utc_timestamp();
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.len(), 20, "{ts}");
        let compact = utc_timestamp_compact();
        assert_eq!(compact.len(), 16, "{compact}");
        let _ = json!({});
    }
}
