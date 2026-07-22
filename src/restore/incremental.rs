use anyhow::Result;

use crate::block::device::BlockDevice;
use crate::repository::layout::Repository;

pub struct IncrementalRestore;

impl IncrementalRestore {
    pub fn execute(
        &self,
        repo: &Repository,
        snapshot_id: &str,
        device: &BlockDevice,
    ) -> Result<()> {
        let chain = repo.build_restore_chain(snapshot_id)?;

        log::info!(
            "Starting incremental restore with {} snapshots in chain",
            chain.len()
        );

        for snap_id in &chain {
            let blocks = repo.read_blocks_map(snap_id)?;
            log::info!("Applying snapshot {} ({} blocks)", snap_id, blocks.len());

            let mut block_nums: Vec<u64> = blocks.keys().copied().collect();
            block_nums.sort();

            for block_num in &block_nums {
                if let Some(data) = blocks.get(block_num) {
                    device.write_block(*block_num, data)?;
                }
            }
        }

        log::info!("Incremental restore completed");
        Ok(())
    }
}
