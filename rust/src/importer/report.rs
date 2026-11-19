//! The import summary: what changed, what was skipped, and why.
//!
//! Printed to the terminal and written as `report.json` next to the backup, so
//! every overwrite is auditable and reversible.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::importer::crafting::CraftingStats;
use crate::importer::items::{Plan, SkippedItem};
use crate::importer::mapping::Table;
use crate::importer::store::OverwriteStats;

#[derive(Debug, Default)]
pub struct ImportReport {
    pub command: String,
    pub created_at: String,
    pub dump_items: usize,
    /// Per-table counts: (old, new).
    pub counts: HashMap<&'static str, (usize, usize)>,
    pub stats: HashMap<&'static str, OverwriteStats>,
    pub crafting: Option<CraftingStats>,
    pub skipped: Vec<SkippedItem>,
    pub warnings: Vec<String>,
    pub unmapped: Vec<String>,
    pub missing_item_power: usize,
}

impl ImportReport {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            created_at: utc_timestamp(),
            ..Self::default()
        }
    }

    pub fn warn(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    pub fn record_plan(&mut self, plan: &Plan) {
        self.skipped = plan.skipped.clone();
        self.missing_item_power = plan.missing_item_power;
        self.unmapped = plan
            .items
            .iter()
            .filter(|i| i.unmapped_subcategory)
            .map(|i| format!("{} ({})", i.name, i.unique_name))
            .collect();
    }

    pub fn record_table(
        &mut self,
        table: Table,
        old_count: usize,
        new_count: usize,
        stats: OverwriteStats,
    ) {
        self.counts.insert(table.label(), (old_count, new_count));
        self.stats.insert(table.label(), stats);
    }

    pub fn print(&self) {
        println!();
        println!(
            "{:<14} {:>6} {:>6} {:>6} {:>8} {:>8}",
            "table", "old", "new", "added", "updated", "same"
        );
        let mut keys: Vec<&'static str> = self.counts.keys().copied().collect();
        keys.sort();
        for key in &keys {
            let (old, new) = self.counts[*key];
            let stats = self.stats.get(*key).cloned().unwrap_or_default();
            println!(
                "{:<14} {:>6} {:>6} {:>6} {:>8} {:>8}",
                key, old, new, stats.inserted, stats.updated, stats.unchanged
            );
        }

        for key in &keys {
            let stats = self.stats.get(*key).cloned().unwrap_or_default();
            if !stats.renames.is_empty() {
                println!("\n{}: {} renamed", key, stats.renames.len());
                for (old, new) in stats.renames.iter().take(10) {
                    println!("    {old} -> {new}");
                }
            }
            if !stats.identifier_fixed.is_empty() {
                println!(
                    "{}: {} identifier(s) corrected",
                    key,
                    stats.identifier_fixed.len()
                );
                for (old, new) in stats.identifier_fixed.iter().take(10) {
                    println!("    {old:?} -> {new}");
                }
            }
            if !stats.item_power_changed.is_empty() {
                println!(
                    "{}: {} item power value(s) changed",
                    key,
                    stats.item_power_changed.len()
                );
            }
            if stats.carry_over_without_match > 0 {
                println!(
                    "warning: {} {} row(s) are not in the dump and were kept",
                    stats.carry_over_without_match, key
                );
            }
        }

        if let Some(crafting) = &self.crafting {
            println!(
                "\ncrafting: {} consumable(s), {} added, {} updated, {} unchanged",
                crafting.by_consumable, crafting.inserted, crafting.updated, crafting.unchanged
            );
            if crafting.kept_without_match > 0 {
                println!(
                    "warning: {} crafting row(s) have no dump counterpart and were kept",
                    crafting.kept_without_match
                );
            }
        }

        if !self.unmapped.is_empty() {
            println!(
                "\n{} item(s) have no matching wiki subcategory (written with subcategory_id = null):",
                self.unmapped.len()
            );
            for name in self.unmapped.iter().take(10) {
                println!("    {name}");
            }
            if self.unmapped.len() > 10 {
                println!("    … {} more (see report.json)", self.unmapped.len() - 10);
            }
        }

        if !self.skipped.is_empty() {
            println!(
                "\n{} item(s) skipped (vanity/test/duplicates):",
                self.skipped.len()
            );
            for skipped in self.skipped.iter().take(6) {
                println!("    {}: {}", skipped.name, skipped.reason);
            }
            if self.skipped.len() > 6 {
                println!("    … {} more (see report.json)", self.skipped.len() - 6);
            }
        }

        if self.missing_item_power > 0 {
            println!(
                "\nwarning: {} item(s) have no item power in the dump or the dataset",
                self.missing_item_power
            );
        }
        for warning in &self.warnings {
            println!("warning: {warning}");
        }

        println!(
            "\nnote: the dump has no per-quality stats, spells or wiki paths — \
             new items have none of those until a permitted source is available."
        );
    }

    pub fn write_json(&self, dir: &Path) -> io::Result<PathBuf> {
        fs::create_dir_all(dir)?;
        let stats_json: Vec<Value> = {
            let mut keys: Vec<&'static str> = self.counts.keys().copied().collect();
            keys.sort();
            keys.into_iter()
                .map(|key| {
                    let stats = self.stats.get(key).cloned().unwrap_or_default();
                    let (old, new) = self.counts[key];
                    json!({
                        "table": key,
                        "old_count": old,
                        "new_count": new,
                        "inserted": stats.inserted,
                        "updated": stats.updated,
                        "unchanged": stats.unchanged,
                        "renamed": stats.renames,
                        "identifiers_fixed": stats.identifier_fixed,
                        "item_power_changed": stats.item_power_changed,
                        "tier_changed": stats.tier_changed,
                        "kept_not_in_dump": stats.carry_over_without_match,
                    })
                })
                .collect()
        };
        let crafting = self.crafting.as_ref().map(|c| {
            json!({
                "consumables": c.by_consumable,
                "inserted": c.inserted,
                "updated": c.updated,
                "unchanged": c.unchanged,
                "kept_not_in_dump": c.kept_without_match,
            })
        });
        let value = json!({
            "command": self.command,
            "created_at": self.created_at,
            "dump_items": self.dump_items,
            "tables": stats_json,
            "crafting": crafting,
            "skipped": self.skipped.iter().map(|s| json!({
                "unique_name": s.unique_name, "name": s.name, "reason": s.reason,
            })).collect::<Vec<Value>>(),
            "unmapped_subcategory": self.unmapped,
            "missing_item_power": self.missing_item_power,
            "warnings": self.warnings,
        });
        let path = dir.join("report.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&value).map_err(io::Error::other)?,
        )?;
        Ok(path)
    }
}

/// `YYYYMMDDTHHMMSSZ` for the backup folder name.
pub fn utc_stamp() -> String {
    let (y, mo, d, h, mi, s) = utc_parts();
    format!("{y:04}{mo:02}{d:02}T{h:02}{mi:02}{s:02}Z")
}

pub fn utc_timestamp() -> String {
    let (y, mo, d, h, mi, s) = utc_parts();
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
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

    #[test]
    fn stamps_are_utc_and_well_formed() {
        let stamp = utc_stamp();
        assert_eq!(stamp.len(), 16, "{stamp}");
        assert!(stamp.ends_with('Z'));
        let ts = utc_timestamp();
        assert_eq!(ts.len(), 20, "{ts}");
    }

    #[test]
    fn report_json_has_counts_and_details() {
        let dir =
            std::env::temp_dir().join(format!("openalbion-import-report-{}", std::process::id()));
        let mut report = ImportReport::new("import");
        let stats = OverwriteStats {
            inserted: 2,
            renames: vec![("Old".into(), "New".into())],
            ..OverwriteStats::default()
        };
        report.record_table(Table::Weapons, 10, 12, stats);
        report.warn("something");
        let path = report.write_json(&dir).unwrap();
        let value: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(value["tables"][0]["old_count"], 10);
        assert_eq!(value["tables"][0]["new_count"], 12);
        assert_eq!(value["tables"][0]["inserted"], 2);
        assert_eq!(value["tables"][0]["renamed"][0][1], "New");
        assert_eq!(value["warnings"][0], "something");
        let _ = fs::remove_dir_all(dir);
    }
}
