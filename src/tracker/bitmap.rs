use anyhow::Result;
use std::collections::BTreeSet;
use std::sync::Mutex;

use super::{ChangeTracker, Checkpoint};

pub struct BitmapTracker {
    #[allow(dead_code)]
    block_size: u32,
    dirty_blocks: Mutex<BTreeSet<u64>>,
    total_blocks: Mutex<u64>,
    checkpoints: Mutex<Vec<Checkpoint>>,
}

impl BitmapTracker {
    pub fn new(block_size: u32) -> Result<Self> {
        Ok(Self {
            block_size,
            dirty_blocks: Mutex::new(BTreeSet::new()),
            total_blocks: Mutex::new(0),
            checkpoints: Mutex::new(Vec::new()),
        })
    }

    pub fn mark_dirty(&self, block_num: u64) {
        if let Ok(mut blocks) = self.dirty_blocks.lock() {
            blocks.insert(block_num);
        }
    }

    pub fn mark_range_dirty(&self, start: u64, end: u64) {
        if let Ok(mut blocks) = self.dirty_blocks.lock() {
            for b in start..=end {
                blocks.insert(b);
            }
        }
    }

    pub fn set_total_blocks(&self, count: u64) {
        if let Ok(mut total) = self.total_blocks.lock() {
            *total = count;
        }
    }
}

impl ChangeTracker for BitmapTracker {
    fn get_changed_blocks(&self, _since: Option<Checkpoint>) -> Result<Vec<u64>> {
        let blocks = self
            .dirty_blocks
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let result: Vec<u64> = blocks.iter().copied().collect();
        Ok(result)
    }

    fn create_checkpoint(&self) -> Result<Checkpoint> {
        let checkpoint = Checkpoint {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            block_count: {
                let blocks = self
                    .dirty_blocks
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
                blocks.len() as u64
            },
            tracking_method: "bitmap".to_string(),
        };

        if let Ok(mut cps) = self.checkpoints.lock() {
            cps.push(checkpoint.clone());
        }

        // Clear dirty blocks after checkpoint
        if let Ok(mut blocks) = self.dirty_blocks.lock() {
            blocks.clear();
        }

        Ok(checkpoint)
    }

    fn reset_tracking(&self) -> Result<()> {
        if let Ok(mut blocks) = self.dirty_blocks.lock() {
            blocks.clear();
        }
        Ok(())
    }

    fn get_current_checkpoint(&self) -> Result<Option<Checkpoint>> {
        let cps = self
            .checkpoints
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        Ok(cps.last().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmap_basic() {
        let tracker = BitmapTracker::new(4096).unwrap();
        tracker.mark_dirty(10);
        tracker.mark_dirty(20);
        tracker.mark_dirty(30);

        let blocks = tracker.get_changed_blocks(None).unwrap();
        assert_eq!(blocks.len(), 3);
        assert!(blocks.contains(&10));
    }

    #[test]
    fn test_checkpoint_clears() {
        let tracker = BitmapTracker::new(4096).unwrap();
        tracker.mark_dirty(5);
        let cp = tracker.create_checkpoint().unwrap();
        assert_eq!(cp.block_count, 1);

        let blocks = tracker.get_changed_blocks(None).unwrap();
        assert!(blocks.is_empty());
    }
}
