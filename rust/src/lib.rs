//! OpenAlbion API library — everything the binary and the integration
//! tests build on.

pub mod api;
pub mod dto;
pub mod models;
pub mod render;
pub mod state;
pub mod util;
pub mod web;

pub use state::{AppConfig, AppState};
