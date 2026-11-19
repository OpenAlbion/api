//! `ao-bin-dumps` importer.
//!
//! Overwrites the item tables from the public game-data dump
//! (<https://github.com/ao-data/ao-bin-dumps>) — no crawling involved — with
//! the old dataset preserved in `data/backups/<timestamp>/`.
//!
//! Modules:
//! - [`aod`]       — download/parse the dump into normalized items
//! - [`mapping`]   — classification and category/subcategory mapping
//! - [`items`]     — plan + overwrite of the four item tables
//! - [`crafting`]  — regenerate `consumable_craftings`
//! - [`store`]     — dataset rows, JSON format, backup
//! - [`report`]    — the run summary and `report.json`

pub mod aod;
pub mod crafting;
pub mod items;
pub mod mapping;
pub mod report;
pub mod store;

pub use store::Dataset;
