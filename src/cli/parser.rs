use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct ParsedArgs {
    pub source: Option<String>,
    pub destination: Option<String>,
    pub config_path: Option<PathBuf>,
    pub block_size: u32,
    pub compression_level: i32,
    pub encryption_key: Option<String>,
    pub verbose: bool,
}

impl Default for ParsedArgs {
    fn default() -> Self {
        Self {
            source: None,
            destination: None,
            config_path: None,
            block_size: 4096,
            compression_level: 3,
            encryption_key: None,
            verbose: false,
        }
    }
}

pub fn parse_block_size(size_str: &str) -> Result<u32> {
    let size = size_str
        .parse::<u32>()
        .context("Invalid block size: must be a positive integer")?;

    if !size.is_power_of_two() {
        anyhow::bail!("Block size must be a power of two (e.g., 512, 4096, 8192)");
    }

    if size < 512 || size > 1_048_576 {
        anyhow::bail!("Block size must be between 512 bytes and 1 MiB");
    }

    Ok(size)
}

pub fn parse_size_human(size: &str) -> Result<u64> {
    let size = size.trim().to_lowercase();

    let multipliers = [
        ("ki", 1024u64),
        ("mi", 1024u64.pow(2)),
        ("gi", 1024u64.pow(3)),
        ("ti", 1024u64.pow(4)),
        ("k", 1000u64),
        ("m", 1000u64.pow(2)),
        ("g", 1000u64.pow(3)),
        ("t", 1000u64.pow(4)),
        ("kb", 1000u64),
        ("mb", 1000u64.pow(2)),
        ("gb", 1000u64.pow(3)),
        ("tb", 1000u64.pow(4)),
    ];

    for (suffix, multiplier) in &multipliers {
        if let Some(num_str) = size.strip_suffix(suffix) {
            let num: f64 = num_str.trim().parse()?;
            return Ok((num * *multiplier as f64) as u64);
        }
    }

    let num: u64 = size.parse()?;
    Ok(num)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_block_sizes() {
        assert!(parse_block_size("4096").is_ok());
        assert!(parse_block_size("512").is_ok());
        assert!(parse_block_size("65536").is_ok());
    }

    #[test]
    fn test_invalid_block_sizes() {
        assert!(parse_block_size("4097").is_err());
        assert!(parse_block_size("0").is_err());
        assert!(parse_block_size("abc").is_err());
    }

    #[test]
    fn test_human_sizes() {
        assert_eq!(parse_size_human("1K").unwrap(), 1000);
        assert_eq!(parse_size_human("1Ki").unwrap(), 1024);
        assert_eq!(parse_size_human("1M").unwrap(), 1_000_000);
        assert_eq!(parse_size_human("1Mi").unwrap(), 1_048_576);
        assert_eq!(parse_size_human("1G").unwrap(), 1_000_000_000);
        assert_eq!(parse_size_human("1Gi").unwrap(), 1_073_741_824);
    }
}
