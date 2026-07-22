use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::block::device::BlockDevice;
use crate::block::hasher::BlockHasher;
use crate::repository::layout::Repository;

pub struct FullBackup {
    block_size: u32,
    hasher: BlockHasher,
}

impl FullBackup {
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
        total_blocks: u64,
    ) -> Result<()> {
        #[cfg(feature = "progress")]
        let pb = {
            let pb = ProgressBar::new(total_blocks);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} blocks ({eta})")?
                    .progress_chars("#>-"),
            );
            pb
        };

        let all_blocks: Vec<(u64, Vec<u8>)> = (0..total_blocks)
            .into_par_iter()
            .map(|block_num| {
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

        #[cfg(feature = "progress")]
        pb.finish_with_message("Full backup complete");

        repo.finalize_snapshot(snapshot_id)?;
        Ok(())
    }
}
