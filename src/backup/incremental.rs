use anyhow::Result;
use rayon::prelude::*;

use crate::block::device::BlockDevice;
use crate::block::hasher::BlockHasher;
use crate::repository::layout::Repository;

pub struct IncrementalBackup {
    #[allow(dead_code)]
    block_size: u32,
    hasher: BlockHasher,
}

impl IncrementalBackup {
    pub fn new(block_size: u32) -> Self {
        Self {
            block_size,
            hasher: BlockHasher::new(),
        }
    }

    pub async fn execute(
        &self,
        device: &BlockDevice,
        repo: &Repository,
        snapshot_id: &str,
        changed_blocks: &[u64],
    ) -> Result<()> {
        if changed_blocks.is_empty() {
            log::info!("No changed blocks to back up");
            return Ok(());
        }

        log::info!("Processing {} changed blocks", changed_blocks.len());

        let all_blocks: Vec<(u64, Vec<u8>)> = changed_blocks
            .par_iter()
            .map(|&block_num| {
                let data = device.read_block(block_num)?;
                Ok((block_num, data))
            })
            .collect::<Result<Vec<_>>>()?;

        let all_hashes: Vec<_> = all_blocks
            .par_iter()
            .map(|(block_num, data)| self.hasher.hash_block_with_number(*block_num, data))
            .collect();

        repo.write_blocks(snapshot_id, &all_blocks)?;
        repo.store_block_hashes(snapshot_id, &all_hashes)?;

        repo.finalize_snapshot(snapshot_id)?;
        log::info!("Incremental backup completed: {} blocks", changed_blocks.len());
        Ok(())
    }
}
