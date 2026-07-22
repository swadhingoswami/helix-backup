use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BlockHash {
    pub block_number: u64,
    pub hash: [u8; 32],
    pub block_size: u32,
}

pub struct BlockHasher;

impl BlockHasher {
    pub fn new() -> Self {
        Self
    }

    pub fn hash_block(&self, data: &[u8]) -> BlockHash {
        let hash = blake3::hash(data);
        BlockHash {
            block_number: 0,
            hash: *hash.as_bytes(),
            block_size: data.len() as u32,
        }
    }

    pub fn hash_block_with_number(&self, block_number: u64, data: &[u8]) -> BlockHash {
        let hash = blake3::hash(data);
        BlockHash {
            block_number,
            hash: *hash.as_bytes(),
            block_size: data.len() as u32,
        }
    }

    pub fn verify_block(&self, data: &[u8], expected: &BlockHash) -> bool {
        let computed = blake3::hash(data);
        computed.as_bytes() == &expected.hash
    }

    pub fn hash_raw(data: &[u8]) -> blake3::Hash {
        blake3::hash(data)
    }
}

pub fn compute_checksum(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hash.to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_consistency() {
        let hasher = BlockHasher::new();
        let data = b"Hello, Helix!";
        let hash1 = hasher.hash_block(data);
        let hash2 = hasher.hash_block(data);
        assert_eq!(hash1.hash, hash2.hash);
    }

    #[test]
    fn test_hash_different_data() {
        let hasher = BlockHasher::new();
        let h1 = hasher.hash_block(b"data1");
        let h2 = hasher.hash_block(b"data2");
        assert_ne!(h1.hash, h2.hash);
    }

    #[test]
    fn test_verify_block() {
        let hasher = BlockHasher::new();
        let data = b"verify me";
        let hash = hasher.hash_block(data);
        assert!(hasher.verify_block(data, &hash));
        assert!(!hasher.verify_block(b"wrong", &hash));
    }

    #[test]
    fn test_checksum_format() {
        let cs = compute_checksum(b"test");
        assert_eq!(cs.len(), 64);
        assert!(cs.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
