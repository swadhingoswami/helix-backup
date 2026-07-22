use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::Config;

pub fn load_config(path: Option<&str>) -> Result<Config> {
    if let Some(p) = path {
        return load_from_file(p);
    }

    let config_dirs = config_file_paths();
    for dir in &config_dirs {
        if dir.exists() {
            if let Some(s) = dir.to_str() {
                return load_from_file(s);
            }
        }
    }

    if let Ok(env_config) = load_from_env() {
        return Ok(env_config);
    }

    Ok(Config::default())
}

fn config_file_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        for name in &["helix.yaml", "helix.yml", "config.yaml", "helix.config.yaml"] {
            paths.push(cwd.join(name));
        }
    }

    if let Some(home) = dirs_config_dir() {
        paths.push(home.join("helix").join("config.yaml"));
    }

    paths.push(PathBuf::from("/etc/helix/config.yaml"));

    paths
}

fn dirs_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })
    }

    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

fn load_from_file(path: &str) -> Result<Config> {
    let path = Path::new(path);
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: Config = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    log::info!("Loaded configuration from {}", path.display());
    Ok(config)
}

fn load_from_env() -> Result<Config> {
    let mut config = Config::default();

    if let Ok(val) = std::env::var("HELIX_BLOCK_SIZE") {
        config.block_size = val.parse()?;
    }
    if let Ok(val) = std::env::var("HELIX_COMPRESSION_LEVEL") {
        config.compression_level = val.parse()?;
    }
    if let Ok(val) = std::env::var("HELIX_REPOSITORY_PATH") {
        config.storage.repository_path = val;
    }
    if let Ok(val) = std::env::var("HELIX_ENCRYPTION_KEY") {
        config.encryption.enabled = true;
        config.encryption.key_path = Some(val);
    }
    if let Ok(val) = std::env::var("HELIX_LOG_LEVEL") {
        config.logging.level = val;
    }

    Ok(config)
}

pub fn save_config(config: &Config, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_yaml::to_string(config)?;
    std::fs::write(path, content)?;

    log::info!("Configuration saved to {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config_loads() {
        let config = load_config(None).unwrap();
        assert_eq!(config.block_size, 4096);
        assert_eq!(config.compression_level, 3);
    }

    #[test]
    fn test_load_from_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test_config.yaml");

        let test_config = Config::default();
        save_config(&test_config, &path).unwrap();

        let loaded = load_config(Some(path.to_str().unwrap())).unwrap();
        assert_eq!(loaded.block_size, 4096);
    }

    #[test]
    fn test_invalid_config_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("invalid.yaml");
        std::fs::write(&path, "invalid: [yaml: content").unwrap();
        assert!(load_config(Some(path.to_str().unwrap())).is_err());
    }
}
