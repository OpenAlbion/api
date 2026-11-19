//! `openalbion-import` — refresh the item dataset from `ao-bin-dumps`.
//!
//! The public game-data dump (<https://github.com/ao-data/ao-bin-dumps>) is
//! authoritative for item data: every run snapshots the current dataset into
//! `data/backups/<timestamp>/` and then **overwrites** the item tables and the
//! consumable crafting recipes with the dump's values. Wiki-only fields
//! (`path`, `info`) and downstream data (stats, spells) are preserved.
//!
//! Exit codes: `0` clean, `2` completed with warnings, `1` fatal.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use openalbion_api::importer::aod;
use openalbion_api::importer::crafting;
use openalbion_api::importer::items;
use openalbion_api::importer::mapping::Table;
use openalbion_api::importer::report::{utc_stamp, ImportReport};
use openalbion_api::importer::store::{backup, Dataset, TableFile};

#[derive(Parser)]
#[command(
    name = "openalbion-import",
    version,
    about = "Refresh the OpenAlbion item dataset from ao-bin-dumps",
    long_about = "Downloads the public ao-bin-dumps game-data dump and overwrites the item \
                  tables (weapons, armors, accessories, consumables) plus the consumable \
                  crafting recipes. The previous dataset is always copied to \
                  data/backups/<timestamp>/ first, so every overwrite is reversible.\n\n\
                  Wiki-only fields (paths, consumable descriptions) and downstream data \
                  (stats, spells) are preserved. Use --dry-run to inspect the diff first."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Dataset directory (the API reads the same files).
    #[arg(long, env = "DATA_DIR", default_value = "data")]
    data_dir: PathBuf,

    /// Use a local checkout of ao-bin-dumps instead of downloading.
    #[arg(long)]
    from_dir: Option<PathBuf>,

    /// Re-download the dump instead of using the cached copy.
    #[arg(long)]
    refresh: bool,

    /// Print the report without writing anything.
    #[arg(long)]
    dry_run: bool,

    /// Also import cosmetics/founder/starter-pack items (skipped by default).
    #[arg(long)]
    include_cosmetics: bool,

    /// Only import items or only crafting.
    #[arg(long, value_enum)]
    only: Option<Only>,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Only {
    Items,
    Crafting,
}

#[derive(Subcommand)]
enum Command {
    /// Process the dump (default when no subcommand is given).
    Import,
    /// Show the dataset and dump counts without changing anything.
    Status,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli).await {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<ExitCode, String> {
    let data_dir = cli.data_dir.clone();
    let mut dataset = Dataset::load(&data_dir).map_err(|e| e.to_string())?;

    println!(
        "loading ao-bin-dumps{}…",
        if cli.refresh { " (refreshing)" } else { "" }
    );
    let dump = aod::load(&data_dir, cli.from_dir.as_deref(), cli.refresh)
        .await
        .map_err(|e| e.to_string())?;
    if dump.is_empty() {
        return Err("the dump contained no usable items — nothing written".to_string());
    }
    println!("dump: {} item definitions", dump.len());

    if matches!(cli.command, Some(Command::Status)) {
        println!("\ndataset vs dump:");
        let index = dataset.identity_index();
        for table in [Table::Weapons, Table::Armors, Table::Accessories] {
            let rows = dataset.items(table);
            let matched = rows
                .iter()
                .filter(|r| {
                    index
                        .find(r.identifier.as_deref().unwrap_or(""), &r.name)
                        .is_some()
                })
                .count();
            println!(
                "  {:<12} {:>5} rows ({:>5} matched in dump)",
                table.label(),
                rows.len(),
                matched
            );
        }
        println!(
            "  {:<12} {:>5} rows",
            "consumables",
            dataset.consumables.len()
        );
        println!(
            "  {:<12} {:>5} rows",
            "crafting",
            dataset.consumable_craftings.len()
        );
        return Ok(ExitCode::SUCCESS);
    }

    let old_counts = counts(&dataset);
    let plan = items::plan(&dump, &dataset, cli.include_cosmetics);
    let mut report = ImportReport::new("import");
    report.dump_items = dump.len();
    report.record_plan(&plan);

    println!(
        "\nplanned: {} weapons, {} armors, {} accessories, {} consumables ({} skipped)",
        plan.count(Table::Weapons),
        plan.count(Table::Armors),
        plan.count(Table::Accessories),
        plan.count(Table::Consumables),
        plan.skipped.len()
    );

    let import_items = !matches!(cli.only, Some(Only::Crafting));
    let import_crafting = !matches!(cli.only, Some(Only::Items));

    let mut changed_files: Vec<TableFile> = Vec::new();
    if import_items {
        let stats = items::apply(&plan, &mut dataset);
        let new_counts = counts(&dataset);
        for table in [
            Table::Weapons,
            Table::Armors,
            Table::Accessories,
            Table::Consumables,
        ] {
            let table_stats = stats.get(&table).cloned().unwrap_or_default();
            if table_stats.changed() || table_stats.carry_over_without_match > 0 {
                changed_files.push(TableFile::of(table));
            }
            if table_stats.carry_over_without_match > 0 {
                report.warn(format!(
                    "{} row(s) of {} are not in the dump and were kept",
                    table_stats.carry_over_without_match,
                    table.label()
                ));
            }
            report.record_table(
                table,
                old_counts.get(table.label()).copied().unwrap_or(0),
                new_counts.get(table.label()).copied().unwrap_or(0),
                table_stats,
            );
        }
        // A table the dump could not describe at all must not be wiped.
        for table in [Table::Weapons, Table::Armors, Table::Accessories] {
            if plan.count(table) == 0 && !dataset.items(table).is_empty() {
                report.warn(format!(
                    "the dump produced no {} rows — that table was left untouched",
                    table.label()
                ));
                dataset = restore_table(dataset, &data_dir, table)?;
                changed_files.retain(|f| *f != TableFile::of(table));
            }
        }
    }

    if import_crafting {
        let consumables: Vec<(i64, String, String)> = dataset
            .consumables
            .iter()
            .map(|row| {
                (
                    row.id,
                    row.identifier.clone().unwrap_or_default(),
                    row.name.clone(),
                )
            })
            .collect();
        let next_id = crafting::next_crafting_id(&dataset.consumable_craftings);
        let (rows, stats) =
            crafting::regenerate(&dump, &dataset.consumable_craftings, &consumables, next_id);
        if stats.changed() || stats.kept_without_match > 0 {
            changed_files.push(TableFile::ConsumableCraftings);
        }
        if stats.kept_without_match > 0 {
            report.warn(format!(
                "{} crafting row(s) have no dump counterpart and were kept",
                stats.kept_without_match
            ));
        }
        dataset.consumable_craftings = rows;
        report.crafting = Some(stats);
    }

    changed_files.sort();
    changed_files.dedup();

    if changed_files.is_empty() {
        println!("\nno changes — dataset already matches the dump");
        report.print();
        return Ok(if report.warnings.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(2)
        });
    }

    if cli.dry_run {
        println!("\ndry run — these files would be written:");
        for file in &changed_files {
            println!("    {}", file.file());
        }
        report.print();
        return Ok(ExitCode::SUCCESS);
    }

    let stamp = utc_stamp();
    let backup_dir = backup(&data_dir, &stamp).map_err(|e| e.to_string())?;
    dataset
        .save(&data_dir, &changed_files)
        .map_err(|e| e.to_string())?;
    report.print();
    let target = backup_dir.unwrap_or_else(|| data_dir.join("backups").join(&stamp));
    let report_path = report.write_json(&target).map_err(|e| e.to_string())?;
    println!(
        "\nwrote: {}",
        changed_files
            .iter()
            .map(|f| f.file())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("backup + report: {}", report_path.display());

    Ok(if report.warnings.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    })
}

fn counts(dataset: &Dataset) -> std::collections::HashMap<&'static str, usize> {
    let mut map = std::collections::HashMap::new();
    map.insert("weapons", dataset.weapons.len());
    map.insert("armors", dataset.armors.len());
    map.insert("accessories", dataset.accessories.len());
    map.insert("consumables", dataset.consumables.len());
    map
}

/// Re-reads one item table so a skipped overwrite really leaves it untouched.
fn restore_table(
    mut dataset: Dataset,
    data_dir: &std::path::Path,
    table: Table,
) -> Result<Dataset, String> {
    let reloaded = Dataset::load(data_dir).map_err(|e| e.to_string())?;
    match table {
        Table::Weapons => dataset.weapons = reloaded.weapons,
        Table::Armors => dataset.armors = reloaded.armors,
        Table::Accessories => dataset.accessories = reloaded.accessories,
        Table::Consumables => dataset.consumables = reloaded.consumables,
    }
    Ok(dataset)
}
