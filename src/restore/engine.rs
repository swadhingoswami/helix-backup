use anyhow::Result;

use crate::block::device::BlockDevice;
use crate::repository::layout::Repository;

pub struct RestoreEngine;

impl RestoreEngine {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub async fn run_restore(&self, source: &str, target: &str, point: Option<&str>) -> Result<()> {
        let repo = Repository::open(source)?;
        let device = BlockDevice::open_for_write(target, repo.block_size())?;

        match point {
            Some("latest") | None => {
                let snapshots = repo.list_backups()?;
                let latest = snapshots
                    .last()
                    .ok_or_else(|| anyhow::anyhow!("No backup snapshots found"))?;

                if latest.backup_type == "full" {
                    let restore = crate::restore::full::FullRestore;
                    restore.execute(&repo, &latest.id, &device)?;
                } else if latest.backup_type == "incremental" {
                    let restore = crate::restore::incremental::IncrementalRestore;
                    restore.execute(&repo, &latest.id, &device)?;
                }
            }
            Some(id) => {
                let snapshots = repo.list_backups()?;
                let snap = snapshots
                    .iter()
                    .find(|s| s.id == id)
                    .ok_or_else(|| anyhow::anyhow!("Snapshot '{}' not found", id))?;

                if snap.backup_type == "full" {
                    let restore = crate::restore::full::FullRestore;
                    restore.execute(&repo, &snap.id, &device)?;
                } else {
                    let restore = crate::restore::incremental::IncrementalRestore;
                    restore.execute(&repo, &snap.id, &device)?;
                }
            }
        }

        log::info!("Restore completed successfully");
        Ok(())
    }
}
