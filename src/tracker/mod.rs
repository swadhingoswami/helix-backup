pub mod bitmap;
pub mod sqlite_store;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub timestamp: i64,
    pub block_count: u64,
    pub tracking_method: String,
}

pub trait ChangeTracker: Send + Sync {
    fn get_changed_blocks(&self, since: Option<Checkpoint>) -> Result<Vec<u64>>;
    fn create_checkpoint(&self) -> Result<Checkpoint>;
    fn reset_tracking(&self) -> Result<()>;
    fn get_current_checkpoint(&self) -> Result<Option<Checkpoint>>;
}

pub fn create_tracker() -> Result<Box<dyn ChangeTracker>> {
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::DmEraTracker::new()?))
    }

    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(macos::FseventsTracker::new()?))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Ok(Box::new(bitmap::BitmapTracker::new(4096)?))
    }
}
