use anyhow::Result;
use std::fs;
use std::path::Path;

use super::{ChangeTracker, Checkpoint};

pub struct DmEraTracker {
    device_path: String,
}

impl DmEraTracker {
    pub fn new() -> Result<Self> {
        Ok(Self {
            device_path: String::new(),
        })
    }

    fn find_dm_era_devices() -> Result<Vec<String>> {
        let mut devices = Vec::new();
        let dm_dir = Path::new("/dev/mapper");

        if dm_dir.exists() {
            if let Ok(entries) = fs::read_dir(dm_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.contains("-era") {
                        devices.push(entry.path().to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(devices)
    }

    fn query_era_blocks(&self, _device: &str) -> Result<Vec<u64>> {
        anyhow::bail!("dm-era query not implemented. Requires kernel support and appropriate permissions.")
    }
}

impl ChangeTracker for DmEraTracker {
    fn get_changed_blocks(&self, _since: Option<Checkpoint>) -> Result<Vec<u64>> {
        let devices = Self::find_dm_era_devices()?;
        if devices.is_empty() {
            log::warn!("No dm-era devices found. Falling back to full scan.");
            return Ok(Vec::new());
        }
        self.query_era_blocks(&devices[0])
    }

    fn create_checkpoint(&self) -> Result<Checkpoint> {
        Ok(Checkpoint {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            block_count: 0,
            tracking_method: "dm-era".to_string(),
        })
    }

    fn reset_tracking(&self) -> Result<()> {
        Ok(())
    }

    fn get_current_checkpoint(&self) -> Result<Option<Checkpoint>> {
        Ok(None)
    }
}
