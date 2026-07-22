use anyhow::Result;
use rayon::prelude::*;

use crate::block::device::BlockDevice;
use crate::block::hasher::BlockHasher;
use crate::repository::layout::Repository;

pub struct BackupEngine {
    block_size: u32,
    #[allow(dead_code)]
    hasher: BlockHasher,
}

impl BackupEngine {
    pub fn new(block_size: u32) -> Result<Self> {
        Ok(Self {
            block_size,
            hasher: BlockHasher::new(),
        })
    }

    pub async fn run_full_backup(&self, source: &str, dest: &str, label: Option<&str>) -> Result<()> {
        let repo = Repository::open_or_create(dest)?;
        let device = BlockDevice::open(source, self.block_size)?;
        let total_blocks = device.block_count()?;

        let backup_label = label.unwrap_or("full-backup");
        let snapshot_id = repo.create_snapshot(backup_label, "full")?;

        log::info!("Starting full backup of {} ({} blocks, {} bytes/block)",
            source, total_blocks, self.block_size);

        let full = crate::backup::full::FullBackup::new(self.block_size);
        full.execute(&device, &repo, &snapshot_id, total_blocks).await?;

        log::info!("Full backup completed: {} blocks backed up", total_blocks);
        Ok(())
    }

    pub async fn run_incremental_backup(&self, source: &str, dest: &str, label: Option<&str>) -> Result<()> {
        let repo = Repository::open(dest)?;
        let device = BlockDevice::open(source, self.block_size)?;

        let changed_blocks = self.detect_changes(&repo, &device)?;

        if changed_blocks.is_empty() {
            log::info!("No changes detected since last backup");
            return Ok(());
        }

        let backup_label = label.unwrap_or("incremental-backup");
        let snapshot_id = repo.create_snapshot(backup_label, "incremental")?;

        log::info!("Starting incremental backup: {} changed blocks", changed_blocks.len());

        let inc = crate::backup::incremental::IncrementalBackup::new(self.block_size);
        inc.execute(&device, &repo, &snapshot_id, &changed_blocks).await?;

        log::info!("Incremental backup completed: {} blocks", changed_blocks.len());
        Ok(())
    }

    fn detect_changes(&self, repo: &Repository, device: &BlockDevice) -> Result<Vec<u64>> {
        let last_checkpoint = repo.last_checkpoint()?;

        let tracker = crate::tracker::create_tracker()?;
        let tracker_blocks = tracker.get_changed_blocks(last_checkpoint)?;

        if !tracker_blocks.is_empty() {
            return Ok(tracker_blocks);
        }

        let backups = repo.list_backups()?;
        let last_full = backups.iter().rev().find(|b| b.backup_type == "full");
        let last_full_id = match last_full {
            Some(snap) => snap.id.clone(),
            None => {
                log::info!("No prior full backup found for comparison");
                return Ok(Vec::new());
            }
        };

        log::info!("Tracker reported no changes, falling back to hash comparison with snapshot {}", last_full_id);

        let manifest = repo.load_manifest(&last_full_id)?;
        if manifest.block_hashes.is_empty() {
            log::info!("Manifest has no block hashes to compare");
            return Ok(Vec::new());
        }

        let total_blocks = device.block_count()?;
        let manifest_hashes: std::collections::HashMap<u64, [u8; 32]> = manifest.block_hashes
            .iter()
            .map(|h| (h.block_number, h.hash))
            .collect();

        let changed: Vec<u64> = (0..total_blocks)
            .into_par_iter()
            .filter_map(|block_num| {
                match device.read_block(block_num) {
                    Ok(data) => {
                        let hash = *blake3::hash(&data).as_bytes();
                        let stored = manifest_hashes.get(&block_num);
                        if stored.is_none_or(|s| *s != hash) {
                            Some(Ok(block_num))
                        } else {
                            None
                        }
                    }
                    Err(e) => Some(Err(e)),
                }
            })
            .collect::<Result<Vec<_>>>()?;

        log::info!("Hash comparison found {} changed blocks", changed.len());
        Ok(changed)
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }
}
