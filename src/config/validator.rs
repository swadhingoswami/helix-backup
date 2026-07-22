use anyhow::{Context, Result};
use std::path::Path;

use super::Config;

pub fn validate_file(path: &str) -> Result<()> {
    let config_path = Path::new(path);
    if !config_path.exists() {
        anyhow::bail!("Configuration file not found: {}", path);
    }

    let content = std::fs::read_to_string(config_path)
        .with_context(|| format!("Cannot read file: {}", path))?;

    if content.trim().is_empty() {
        anyhow::bail!("Configuration file is empty");
    }

    let config: Config = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("YAML parse error: {}", e))?;

    validate(&config)
}

pub fn validate(config: &Config) -> Result<()> {
    let mut errors: Vec<String> = Vec::new();

    if !config.block_size.is_power_of_two() {
        errors.push(format!(
            "Block size {} is not a power of two",
            config.block_size
        ));
    }
    if config.block_size < 512 || config.block_size > 1_048_576 {
        errors.push(format!(
            "Block size {} is outside supported range (512 - 1,048,576)",
            config.block_size
        ));
    }

    if !(1..=22).contains(&config.compression_level) {
        errors.push(format!(
            "Compression level {} is outside valid range (1-22)",
            config.compression_level
        ));
    }

    let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        errors.push(format!(
            "Invalid log level '{}'. Valid options: {:?}",
            config.logging.level, valid_log_levels
        ));
    }

    let valid_tracking = ["dm-era", "fsevents", "bitmap", "manual"];
    if !valid_tracking.contains(&config.tracking.method.as_str()) {
        errors.push(format!(
            "Invalid tracking method '{}'. Valid options: {:?}",
            config.tracking.method, valid_tracking
        ));
    }

    if config.encryption.enabled {
        if config.encryption.key_path.is_none() && config.encryption.kms_provider.is_none() {
            errors.push(
                "Encryption enabled but no key_path or kms_provider configured".to_string(),
            );
        }
        if config.encryption.cipher != "aes-256-gcm" {
            errors.push(format!("Unsupported cipher: {}", config.encryption.cipher));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let msg = errors.join("\n  - ");
        anyhow::bail!("Configuration validation failed:\n  - {}", msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_default_config() {
        let config = Config::default();
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn test_invalid_block_size() {
        let config = Config { block_size: 1234, ..Config::default() };
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_encryption_without_key() {
        let mut config = Config::default();
        config.encryption.enabled = true;
        assert!(validate(&config).is_err());
    }
}
