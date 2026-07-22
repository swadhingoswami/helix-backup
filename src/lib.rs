pub mod cli;
pub mod config;
pub mod backup;
pub mod restore;
pub mod tracker;
pub mod block;
pub mod repository;
pub mod crypto;
pub mod utils;

pub use utils::errors::HelixError;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "helix";
