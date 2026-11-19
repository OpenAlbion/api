//! Wiki data refresh: the Rust port of the Laravel artisan fetch commands.
//!
//! Modules:
//! - [`text`]      — cleaning/derivation helpers shared by all parsers
//! - [`store`]     — the file-backed dataset (load, merge-upsert, save)
//! - [`report`]    — backups and the old-vs-fetched diff report
//! - [`seeds`]     — hard-coded category/potion seeds from the Laravel code
//! - [`parse`]     — DOM parsers (wiki HTML → row drafts)
//! - [`transport`] — page fetching (Cloudflare cookie HTTP, browser, fixtures)
//! - [`aod`]       — Albion Online Data item dump (identifiers, crafting icons)
//! - [`commands`]  — the command implementations

pub mod aod;
pub mod commands;
pub mod parse;
pub mod report;
pub mod seeds;
pub mod store;
pub mod text;
pub mod transport;

pub use store::Tables;
