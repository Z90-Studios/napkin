#![warn(clippy::all, rust_2018_idioms)]

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod app;
mod theme;
pub use app::AtlasApp;
mod widgets;
mod types;