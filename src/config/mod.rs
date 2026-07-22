pub mod loader;
pub mod validator;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_block_size")]
    pub block_size: u32,

    #[serde(default = "default_compression_level")]
    pub compression_level: i32,

    #[serde(default)]
    pub encryption: EncryptionConfig,

    #[serde(default)]
    pub storage: StorageConfig,

    #[serde(default)]
    pub backup: BackupConfig,

    #[serde(default)]
    pub logging: LoggingConfig,

    #[serde(default)]
    pub performance: PerformanceConfig,

    #[serde(default)]
    pub tracking: TrackingConfig,

    #[serde(default)]
    pub restore: RestoreConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            block_size: 4096,
            compression_level: 3,
            encryption: EncryptionConfig::default(),
            storage: StorageConfig::default(),
            backup: BackupConfig::default(),
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
            tracking: TrackingConfig::default(),
            restore: RestoreConfig::default(),
        }
    }
}

fn default_block_size() -> u32 {
    4096
}
fn default_compression_level() -> i32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub key_path: Option<String>,

    #[serde(default = "default_cipher")]
    pub cipher: String,

    #[serde(default)]
    pub kms_provider: Option<String>,
}

fn default_cipher() -> String {
    "aes-256-gcm".to_string()
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            key_path: None,
            cipher: default_cipher(),
            kms_provider: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_repository_path")]
    pub repository_path: String,

    #[serde(default)]
    pub temp_path: Option<String>,

    #[serde(default = "default_max_parallel_io")]
    pub max_parallel_io: usize,

    #[serde(default)]
    pub compression: CompressionConfig,
}

fn default_repository_path() -> String {
    "/var/helix/backups".to_string()
}
fn default_max_parallel_io() -> usize {
    4
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            repository_path: default_repository_path(),
            temp_path: None,
            max_parallel_io: default_max_parallel_io(),
            compression: CompressionConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_compression_level")]
    pub level: i32,

    #[serde(default)]
    pub algorithm: String,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            level: default_compression_level(),
            algorithm: "zstd".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    #[serde(default)]
    pub exclude_paths: Vec<String>,

    #[serde(default)]
    pub include_paths: Vec<String>,

    #[serde(default = "default_retention_days")]
    pub retention_days: u32,

    #[serde(default)]
    pub schedule: Option<ScheduleConfig>,

    #[serde(default)]
    pub pre_backup_hook: Option<String>,

    #[serde(default)]
    pub post_backup_hook: Option<String>,

    #[serde(default = "default_true")]
    pub verify_after_backup: bool,
}

fn default_retention_days() -> u32 {
    30
}
fn default_true() -> bool {
    true
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            exclude_paths: Vec::new(),
            include_paths: Vec::new(),
            retention_days: default_retention_days(),
            schedule: None,
            pre_backup_hook: None,
            post_backup_hook: None,
            verify_after_backup: default_true(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub enabled: bool,
    pub interval: String,
    pub time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,

    #[serde(default)]
    pub file: Option<String>,

    #[serde(default)]
    pub format: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: None,
            format: "text".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    #[serde(default = "default_buffer_size_mb")]
    pub buffer_size_mb: u32,

    #[serde(default = "default_true")]
    pub direct_io: bool,

    #[serde(default)]
    pub throttle_mbps: Option<u32>,

    #[serde(default = "default_thread_count")]
    pub thread_count: usize,
}

fn default_buffer_size_mb() -> u32 {
    64
}
fn default_thread_count() -> usize {
    4
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            buffer_size_mb: default_buffer_size_mb(),
            direct_io: default_true(),
            throttle_mbps: None,
            thread_count: default_thread_count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingConfig {
    #[serde(default = "default_tracking_method")]
    pub method: String,

    #[serde(default = "default_checkpoint_interval")]
    pub checkpoint_interval_secs: u64,

    #[serde(default)]
    pub persist_path: Option<String>,
}

fn default_tracking_method() -> String {
    if cfg!(target_os = "linux") {
        "dm-era".to_string()
    } else if cfg!(target_os = "macos") {
        "fsevents".to_string()
    } else {
        "bitmap".to_string()
    }
}

fn default_checkpoint_interval() -> u64 {
    300
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            method: default_tracking_method(),
            checkpoint_interval_secs: default_checkpoint_interval(),
            persist_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreConfig {
    #[serde(default = "default_true")]
    pub verify_blocks: bool,

    #[serde(default)]
    pub overwrite: bool,

    #[serde(default)]
    pub dry_run: bool,
}

impl Default for RestoreConfig {
    fn default() -> Self {
        Self {
            verify_blocks: default_true(),
            overwrite: false,
            dry_run: false,
        }
    }
}
