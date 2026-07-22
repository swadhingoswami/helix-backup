pub mod backup;
pub mod block;
pub mod cli;
pub mod config;
pub mod crypto;
pub mod repository;
pub mod restore;
pub mod tracker;
pub mod utils;

pub use utils::errors::HelixError;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "helix";
