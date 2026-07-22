use serde::{Deserialize, Serialize};

use crate::block::hasher::BlockHash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub snapshot_id: String,
    pub backup_type: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub completed: bool,
    pub block_size: u32,
    pub block_count: u64,
    pub total_size: u64,
    pub compressed: bool,
    pub encrypted: bool,
    pub label: String,
    pub parent_id: Option<String>,
    pub block_hashes: Vec<BlockHash>,
}

impl Manifest {
    pub fn new(snapshot_id: &str, backup_type: &str, block_size: u32) -> Self {
        Self {
            version: 1,
            snapshot_id: snapshot_id.to_string(),
            backup_type: backup_type.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            completed: false,
            block_size,
            block_count: 0,
            total_size: 0,
            compressed: false,
            encrypted: false,
            label: String::new(),
            parent_id: None,
            block_hashes: Vec::new(),
        }
    }

    pub fn add_block_hash(&mut self, hash: BlockHash) {
        self.block_count += 1;
        self.total_size += hash.block_size as u64;
        self.block_hashes.push(hash);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreChainEntry {
    pub snapshot_id: String,
    pub backup_type: String,
    pub block_count: u64,
    pub sequence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreChain {
    pub entries: Vec<RestoreChainEntry>,
    pub total_blocks: u64,
    pub target_snapshot: String,
}
