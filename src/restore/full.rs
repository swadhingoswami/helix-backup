use anyhow::Result;

use crate::block::device::BlockDevice;
use crate::repository::layout::Repository;

pub struct FullRestore;

impl FullRestore {
    pub fn execute(
        &self,
        repo: &Repository,
        snapshot_id: &str,
        device: &BlockDevice,
    ) -> Result<()> {
        let blocks = repo.read_blocks_map(snapshot_id)?;
        log::info!("Starting full restore: {} blocks", blocks.len());

        let mut block_nums: Vec<u64> = blocks.keys().copied().collect();
        block_nums.sort();

        for block_num in &block_nums {
            if let Some(data) = blocks.get(block_num) {
                device.write_block(*block_num, data)?;
            }
        }

        log::info!("Full restore completed");
        Ok(())
    }
}
