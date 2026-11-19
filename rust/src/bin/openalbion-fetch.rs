//! `openalbion-fetch` — refresh the JSON dataset from the Albion Online Wiki.
//!
//! Ports the Laravel artisan commands in
//! `app/Console/Commands/{Weapon,Armor,Accessory,Consumable,Migrations}`:
//! every run backs up the old dataset, merges the freshly crawled rows,
//! reports what is new/missing, and **never deletes** existing rows.
//!
//! Exit codes: `0` clean, `2` completed with warnings, `1` fatal.

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use clap::{Parser, Subcommand};

use openalbion_api::fetch::aod;
use openalbion_api::fetch::commands::{Options, Runner};
use openalbion_api::fetch::report::utc_timestamp;
use openalbion_api::fetch::transport::{
    auth_path, load_auth, save_auth, wiki, BrowserFetcher, Transport, TransportMode,
};

#[derive(Parser)]
#[command(
    name = "openalbion-fetch",
    version,
    about = "Refresh the OpenAlbion JSON dataset from the Albion Online Wiki",
    long_about = "Ports the Laravel wiki fetch commands. Each run backs up the existing JSON \
                  tables to data/backups/<timestamp>/, merges the crawled rows, prints an \
                  added/missing report and never deletes existing rows.\n\n\
                  The wiki is Cloudflare-protected: run `openalbion-fetch bootstrap` once \
                  (opens a real Chrome window) to save the clearance cookie."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Dataset directory (the API reads the same files).
    #[arg(long, global = true, env = "DATA_DIR", default_value = "data")]
    data_dir: PathBuf,

    /// How pages are fetched: saved Cloudflare cookie over HTTP (default),
    /// through Chrome, or from HTML fixture files.
    #[arg(long, global = true, value_enum, default_value_t = TransportArg::Cookie)]
    transport: TransportArg,

    /// Chrome/Chromium executable for `bootstrap` and `--transport browser`.
    #[arg(long, global = true, env = "CHROME_PATH")]
    chrome: Option<PathBuf>,

    /// Run Chrome headless (Cloudflare usually needs a visible window).
    #[arg(long, global = true)]
    headless: bool,

    /// Parallel page fetches.
    #[arg(long, global = true, default_value_t = 3)]
    concurrency: usize,

    /// Politeness delay between fetches, in milliseconds.
    #[arg(long, global = true, default_value_t = 300)]
    delay_ms: u64,

    /// Stop after N detail pages (debugging).
    #[arg(long, global = true)]
    limit: Option<usize>,

    /// Save every fetched page as HTML (selector debugging / test fixtures).
    #[arg(long, global = true)]
    dump_html: Option<PathBuf>,

    /// Fetch and report, but do not write anything.
    #[arg(long, global = true)]
    dry_run: bool,

    /// Re-download the AOD item dump instead of using the cached copy.
    #[arg(long, global = true)]
    refresh_aod: bool,

    /// Bootstrap the Cloudflare cookie automatically when it is missing.
    #[arg(long, global = true)]
    auto_bootstrap: bool,

    /// Cookie file location (default: <data-dir>/.cache/cloudflare.json).
    #[arg(long, global = true)]
    cookie_file: Option<PathBuf>,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum TransportArg {
    Cookie,
    Browser,
    Fixture,
}

#[derive(Subcommand)]
enum Command {
    /// Solve the wiki's Cloudflare challenge once and save the clearance cookie.
    Bootstrap,
    /// Fetch one URL and report status/title/size (debugging aid).
    Probe {
        /// Full URL or wiki path (e.g. /wiki/Axe).
        url: String,
    },
    /// Re-download the AOD item dump used by identifier fixups.
    AodRefresh,

    /// Crawl the weapon category tree (/wiki/Weapon).
    WeaponCategory,
    /// Seed the armor category tree.
    ArmorCategory,
    /// Seed the accessory category tree (capes, bags).
    AccessoryCategory,
    /// Seed the consumable category tree (foods, potions).
    ConsumableCategory,
    /// Run every category command.
    Categories,

    /// Crawl weapon lists per subcategory.
    Weapon,
    /// Crawl armor lists per subcategory.
    Armor,
    /// Crawl the cape list.
    Cape,
    /// Crawl the bag list.
    Bag,
    /// Crawl the mount list.
    Mount,
    /// Crawl food lists per subcategory.
    Food,
    /// Seed the potion list.
    Potion,

    /// Crawl weapon stat tables.
    WeaponStat,
    /// Crawl armor stat tables.
    ArmorStat,
    /// Crawl accessory/mount stat tables.
    AccessoryStat,
    /// Crawl consumable stat tables.
    ConsumableStat,

    /// Crawl weapon spells.
    WeaponSpell,
    /// Crawl armor spells.
    ArmorSpell,
    /// Crawl accessory (cape/bag) spells.
    AccessorySpell,
    /// Crawl mount spells.
    MountSpell,

    /// Crawl consumable crafting recipes.
    ConsumableCrafting,
    /// Crawl consumable descriptions (the `info` column).
    ConsumableInfo,

    /// Fill missing identifiers from the AOD item dump.
    Identifiers,
    /// Fill missing item power from the Normal/0 stat rows.
    ItemPower,
    /// Fill missing mount tiers from their identifiers.
    MountTier,
    /// Split capes into Normal/Faction subcategories by spell presence.
    CapeSubcategory,
    /// Point every bag at the Normal Bags subcategory.
    BagSubcategory,
    /// Strip `class="…"` attributes from spell descriptions.
    SanitizeSpellDescription,
    /// Re-fetch descriptions flagged `Sanitize Spell Description`.
    UpdateSpellDescription,

    /// Run the whole pipeline (single backup, single report).
    All,
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

    match &cli.command {
        Command::Bootstrap => {
            println!("Opening Chrome to solve the Cloudflare challenge…");
            let browser = BrowserFetcher::launch(cli.chrome.clone(), cli.headless)
                .await
                .map_err(|e| e.to_string())?;
            let auth = browser
                .bootstrap(&wiki(openalbion_api::fetch::seeds::WEAPON_CATEGORY_PAGE))
                .await
                .map_err(|e| e.to_string())?;
            if !auth.has_clearance() {
                return Err("the challenge was not solved (no cf_clearance cookie) — \
                            try again without --headless and make sure Chrome is not blocked"
                    .to_string());
            }
            let path = cli
                .cookie_file
                .clone()
                .unwrap_or_else(|| auth_path(&data_dir));
            save_auth(&path, &auth).map_err(|e| format!("saving {}: {e}", path.display()))?;
            println!("saved Cloudflare clearance to {}", path.display());
            Ok(ExitCode::SUCCESS)
        }
        Command::Probe { url } => {
            let transport = make_transport(&cli, &data_dir, false).await?;
            let url = wiki(url);
            let html = transport.get_html(&url).await.map_err(|e| e.to_string())?;
            if let Some(dir) = &cli.dump_html {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
                let path = dir.join(openalbion_api::fetch::transport::fixture_name(&url));
                std::fs::write(&path, &html).map_err(|e| e.to_string())?;
                println!("saved {}", path.display());
            }
            println!("url:    {url}");
            println!("title:  {}", extract_title(&html).unwrap_or_default());
            println!("length: {} bytes", html.len());
            println!(
                "has wiki content marker: {}",
                html.contains("mw-content-text")
            );
            Ok(ExitCode::SUCCESS)
        }
        Command::AodRefresh => {
            let cache = data_dir.join(".cache").join("items.json");
            let items = aod::load_or_download(&cache, true)
                .await
                .map_err(|e| e.to_string())?;
            println!(
                "AOD item dump: {} entries → {}",
                items.len(),
                cache.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        _ => {
            // Cookie mode with a missing clearance: bootstrap first if asked.
            if matches!(cli.transport, TransportArg::Cookie)
                && cli.auto_bootstrap
                && load_auth(&auth_path(&data_dir))
                    .map_err(|e| e.to_string())?
                    .is_none()
            {
                println!("no Cloudflare clearance yet — bootstrapping…");
                let browser = BrowserFetcher::launch(cli.chrome.clone(), cli.headless)
                    .await
                    .map_err(|e| e.to_string())?;
                let auth = browser
                    .bootstrap(&wiki(openalbion_api::fetch::seeds::WEAPON_CATEGORY_PAGE))
                    .await
                    .map_err(|e| e.to_string())?;
                save_auth(&auth_path(&data_dir), &auth).map_err(|e| e.to_string())?;
            }

            let name = command_name(&cli.command);
            let transport = make_transport(&cli, &data_dir, true).await?;
            let options = Options {
                data_dir: data_dir.clone(),
                concurrency: cli.concurrency,
                delay_ms: cli.delay_ms,
                limit: cli.limit,
                dump_html: cli.dump_html.clone(),
                dry_run: cli.dry_run,
                refresh_aod: cli.refresh_aod,
            };
            let mut runner = Runner::new(options, transport, name).map_err(|e| e.to_string())?;
            println!("[{name}] {} — {}", utc_timestamp(), data_dir.display());
            dispatch(&mut runner, &cli.command).await?;
            let problems = runner.finish(name).map_err(|e| e.to_string())?;
            if problems > 0 {
                eprintln!("{problems} warning(s)/error(s) — review the report above");
                return Ok(ExitCode::from(2));
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

async fn make_transport(
    cli: &Cli,
    data_dir: &std::path::Path,
    allow_missing_auth: bool,
) -> Result<Arc<Transport>, String> {
    let mode = match cli.transport {
        TransportArg::Cookie => TransportMode::Cookie,
        TransportArg::Browser => TransportMode::Browser,
        TransportArg::Fixture => {
            // `--transport fixture` reads from --dump-html's directory.
            let dir = cli
                .dump_html
                .clone()
                .unwrap_or_else(|| PathBuf::from("fixtures"));
            TransportMode::Fixture(dir)
        }
    };
    if matches!(mode, TransportMode::Cookie)
        && !allow_missing_auth
        && load_auth(&auth_path(data_dir))
            .map_err(|e| e.to_string())?
            .is_none()
    {
        return Err(format!(
            "no Cloudflare clearance at {} — run `openalbion-fetch bootstrap` first",
            auth_path(data_dir).display()
        ));
    }
    let transport = Transport::new(
        &mode,
        data_dir,
        cli.chrome.clone(),
        cli.headless,
        cli.cookie_file.clone(),
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(Arc::new(transport))
}

async fn dispatch(runner: &mut Runner, command: &Command) -> Result<(), String> {
    match command {
        Command::WeaponCategory => runner.weapon_category().await,
        Command::ArmorCategory => runner.armor_category(),
        Command::AccessoryCategory => runner.accessory_category(),
        Command::ConsumableCategory => runner.consumable_category(),
        Command::Categories => runner.categories().await,
        Command::Weapon => runner.weapon().await,
        Command::Armor => runner.armor().await,
        Command::Cape => runner.cape().await,
        Command::Bag => runner.bag().await,
        Command::Mount => runner.mount().await,
        Command::Food => runner.food().await,
        Command::Potion => runner.potion(),
        Command::WeaponStat => runner.weapon_stat().await,
        Command::ArmorStat => runner.armor_stat().await,
        Command::AccessoryStat => runner.accessory_stat().await,
        Command::ConsumableStat => runner.consumable_stat().await,
        Command::WeaponSpell => runner.weapon_spell().await,
        Command::ArmorSpell => runner.armor_spell().await,
        Command::AccessorySpell => runner.accessory_spell().await,
        Command::MountSpell => runner.mount_spell().await,
        Command::ConsumableCrafting => runner.consumable_crafting().await,
        Command::ConsumableInfo => runner.consumable_info().await,
        Command::Identifiers => runner.identifiers().await,
        Command::ItemPower => runner.item_power(),
        Command::MountTier => runner.mount_tier(),
        Command::CapeSubcategory => runner.cape_subcategory(),
        Command::BagSubcategory => runner.bag_subcategory(),
        Command::SanitizeSpellDescription => runner.sanitize_spell_description(),
        Command::UpdateSpellDescription => runner.update_spell_description().await,
        Command::All => runner.all().await,
        Command::Bootstrap | Command::Probe { .. } | Command::AodRefresh => unreachable!(),
    }
    Ok(())
}

fn command_name(command: &Command) -> &'static str {
    match command {
        Command::Bootstrap => "bootstrap",
        Command::Probe { .. } => "probe",
        Command::AodRefresh => "aod-refresh",
        Command::WeaponCategory => "weapon-category",
        Command::ArmorCategory => "armor-category",
        Command::AccessoryCategory => "accessory-category",
        Command::ConsumableCategory => "consumable-category",
        Command::Categories => "categories",
        Command::Weapon => "weapon",
        Command::Armor => "armor",
        Command::Cape => "cape",
        Command::Bag => "bag",
        Command::Mount => "mount",
        Command::Food => "food",
        Command::Potion => "potion",
        Command::WeaponStat => "weapon-stat",
        Command::ArmorStat => "armor-stat",
        Command::AccessoryStat => "accessory-stat",
        Command::ConsumableStat => "consumable-stat",
        Command::WeaponSpell => "weapon-spell",
        Command::ArmorSpell => "armor-spell",
        Command::AccessorySpell => "accessory-spell",
        Command::MountSpell => "mount-spell",
        Command::ConsumableCrafting => "consumable-crafting",
        Command::ConsumableInfo => "consumable-info",
        Command::Identifiers => "identifiers",
        Command::ItemPower => "item-power",
        Command::MountTier => "mount-tier",
        Command::CapeSubcategory => "cape-subcategory",
        Command::BagSubcategory => "bag-subcategory",
        Command::SanitizeSpellDescription => "sanitize-spell-description",
        Command::UpdateSpellDescription => "update-spell-description",
        Command::All => "all",
    }
}

fn extract_title(html: &str) -> Option<String> {
    let start = html.find("<title>")? + "<title>".len();
    let end = html[start..].find("</title>")? + start;
    Some(html[start..end].trim().to_string())
}
