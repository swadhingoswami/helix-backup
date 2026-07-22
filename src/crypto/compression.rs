use anyhow::Result;
use std::io::{Read, Write};

pub struct Compressor {
    level: i32,
}

impl Compressor {
    pub fn new(level: i32) -> Self {
        let level = level.clamp(1, 22);
        Self { level }
    }

    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut compressed = Vec::new();
        let mut encoder = zstd::Encoder::new(&mut compressed, self.level)?;
        encoder.write_all(data)?;
        encoder.finish()?;
        Ok(compressed)
    }

    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decompressed = Vec::new();
        let mut decoder = zstd::Decoder::new(data)?;
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    pub fn compress_blocks(&self, blocks: &[Vec<u8>]) -> Result<Vec<(usize, Vec<u8>)>> {
        blocks
            .iter()
            .map(|block| {
                let compressed = self.compress(block)?;
                Ok((block.len(), compressed))
            })
            .collect()
    }

    pub fn level(&self) -> i32 {
        self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let compressor = Compressor::new(3);
        let data =
            b"Hello, Helix compression! This is test data that should compress well. ".repeat(100);

        let compressed = compressor.compress(&data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
        assert!(
            compressed.len() < data.len(),
            "Compression should reduce size"
        );
    }

    #[test]
    fn test_compression_levels() {
        let data = b"a".repeat(10000);

        let fast = Compressor::new(1);
        let max = Compressor::new(22);

        let fast_compressed = fast.compress(&data).unwrap();
        let max_compressed = max.compress(&data).unwrap();

        assert!(
            max_compressed.len() <= fast_compressed.len(),
            "Higher compression should not increase size"
        );
    }
}
