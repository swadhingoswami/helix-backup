# HELIX API Reference

## Core API

### `helix::backup::engine::BackupEngine`

The main engine for orchestrating backup operations.

```rust
impl BackupEngine {
    /// Create a new backup engine with the specified block size
    pub fn new(block_size: u32) -> Result<Self>;

    /// Execute a full backup of a source device to a destination repository
    pub async fn run_full_backup(
        &self,
        source: &str,       // Device path (e.g., /dev/sda)
        dest: &str,         // Repository path
        label: Option<&str>, // Backup label
    ) -> Result<()>;

    /// Execute an incremental backup of changed blocks
    pub async fn run_incremental_backup(
        &self,
        source: &str,
        dest: &str,
        label: Option<&str>,
    ) -> Result<()>;
}
```

### `helix::restore::engine::RestoreEngine`

Handles restore operations from a backup repository.

```rust
impl RestoreEngine {
    /// Create a new restore engine
    pub fn new() -> Result<Self>;

    /// Restore data from a repository to a target device
    pub async fn run_restore(
        &self,
        source: &str,          // Repository path
        target: &str,          // Target device path
        point: Option<&str>,   // Restore point ("latest", snapshot ID, or None)
    ) -> Result<()>;
}
```

### `helix::block::device::BlockDevice`

Low-level block device I/O operations.

```rust
impl BlockDevice {
    /// Open a block device for reading
    pub fn open(path: &str, block_size: u32) -> Result<Self>;

    /// Open a block device for reading and writing
    pub fn open_for_write(path: &str, block_size: u32) -> Result<Self>;

    /// Read a single block from the device
    pub fn read_block(&self, block_num: u64) -> Result<Vec<u8>>;

    /// Write data to a single block on the device
    pub fn write_block(&self, block_num: u64, data: &[u8]) -> Result<()>;

    /// Get the total number of blocks on the device
    pub fn block_count(&self) -> Result<u64>;

    /// Get the block size
    pub fn block_size(&self) -> u32;

    /// Get the total device size in bytes
    pub fn device_size(&self) -> u64;

    /// Flush/sync all pending writes
    pub fn flush(&self) -> Result<()>;
}
```

### `helix::block::hasher::BlockHasher`

Block hashing with blake3 for integrity verification.

```rust
pub struct BlockHash {
    pub block_number: u64,
    pub hash: [u8; 32],
    pub block_size: u32,
}

impl BlockHasher {
    /// Create a new hasher instance
    pub fn new() -> Self;

    /// Hash block data (block_number will be 0)
    pub fn hash_block(&self, data: &[u8]) -> BlockHash;

    /// Hash block data with specific block number
    pub fn hash_block_with_number(&self, block_number: u64, data: &[u8]) -> BlockHash;

    /// Verify block data against an expected hash
    pub fn verify_block(&self, data: &[u8], expected: &BlockHash) -> bool;
}
```

### `helix::tracker::ChangeTracker` (Trait)

Abstract interface for block change detection.

```rust
pub trait ChangeTracker: Send + Sync {
    /// Get blocks that have changed since the given checkpoint
    fn get_changed_blocks(&self, since: Option<Checkpoint>) -> Result<Vec<u64>>;

    /// Create a new checkpoint, resetting the change tracking state
    fn create_checkpoint(&self) -> Result<Checkpoint>;

    /// Reset all tracking state
    fn reset_tracking(&self) -> Result<()>;

    /// Get the current (latest) checkpoint
    fn get_current_checkpoint(&self) -> Result<Option<Checkpoint>>;
}
```

### `helix::repository::layout::Repository`

Backup repository management.

```rust
impl Repository {
    /// Open an existing repository
    pub fn open(path: &str) -> Result<Self>;

    /// Open or create a repository
    pub fn open_or_create(path: &str) -> Result<Self>;

    /// Initialize a new repository
    pub fn initialize(path: &str, key: Option<&str>, compression_level: i32) -> Result<Self>;

    /// Create a new backup snapshot
    pub fn create_snapshot(&self, label: &str, backup_type: &str) -> Result<String>;

    /// Finalize a snapshot after all data is written
    pub fn finalize_snapshot(&self, snapshot_id: &str) -> Result<()>;

    /// Write full backup blocks
    pub fn write_full_blocks(&self, snapshot_id: &str, blocks: &[(u64, Vec<u8>)]) -> Result<()>;

    /// Write incremental backup blocks
    pub fn write_incremental_blocks(&self, snapshot_id: &str, blocks: &[(u64, Vec<u8>)]) -> Result<()>;

    /// Store block hashes in the manifest
    pub fn store_block_hashes(&self, snapshot_id: &str, hashes: &[BlockHash]) -> Result<()>;

    /// Load a snapshot manifest
    pub fn load_manifest(&self, snapshot_id: &str) -> Result<Manifest>;

    /// List all backups in the repository
    pub fn list_backups(&self) -> Result<Vec<BackupSnapshot>>;

    /// Build the restore chain for a given snapshot
    pub fn build_restore_chain(&self, snapshot_id: &str) -> Result<Vec<String>>;

    /// Validate repository integrity
    pub fn validate(&self, repair: bool) -> Result<ValidationResult>;
}
```

## Error Handling

### `helix::utils::errors::HelixError`

Comprehensive error types with recovery guidance.

```rust
pub enum HelixError {
    Device(String),            // Block device operation failed
    Io(std::io::Error),        // I/O error
    Storage(String),           // Storage subsystem error
    Tracking(String),          // Change tracking error
    Encryption(String),        // Encryption/decryption failed
    Configuration(String),     // Invalid configuration
    Repository(String),        // Repository corruption
    Serialization(serde_json::Error),
    Database(rusqlite::Error),
    InvalidInput(String),      // Invalid user input
    NotFound(String),          // Resource not found
    PermissionDenied(String),  // Insufficient permissions
    Unsupported(String),       // Unsupported operation
    Cancelled,                 // Operation was cancelled
    Unknown(String),           // Unknown error
}

impl HelixError {
    /// Returns true if the operation can be retried
    pub fn is_retryable(&self) -> bool;

    /// Returns true if the error is critical
    pub fn is_critical(&self) -> bool;

    /// Returns a user-friendly error message
    pub fn user_message(&self) -> String;
}
```

## Configuration

### `helix::config::Config`

```rust
pub struct Config {
    pub block_size: u32,
    pub compression_level: i32,
    pub encryption: EncryptionConfig,
    pub storage: StorageConfig,
    pub backup: BackupConfig,
    pub logging: LoggingConfig,
    pub performance: PerformanceConfig,
    pub tracking: TrackingConfig,
    pub restore: RestoreConfig,
}
```

### `helix::config::loader`

```rust
/// Load configuration from file, environment, or defaults
pub fn load_config(path: Option<&str>) -> Result<Config>;

/// Save configuration to a file
pub fn save_config(config: &Config, path: &Path) -> Result<()>;
```

### `helix::config::validator`

```rust
/// Validate a configuration file
pub fn validate_file(path: &str) -> Result<()>;

/// Validate a configuration struct
pub fn validate(config: &Config) -> Result<()>;
```

## Crypto API

### `helix::crypto::encryption::Encryptor`

```rust
impl Encryptor {
    /// Create a new encryptor with a 256-bit key
    pub fn new(key: &[u8; 32]) -> Self;

    /// Encrypt data (returns nonce || ciphertext)
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt data (expects nonce || ciphertext)
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Generate a random 256-bit key
    pub fn generate_key() -> [u8; 32];
}
```

### `helix::crypto::compression::Compressor`

```rust
impl Compressor {
    /// Create a new compressor with the specified level
    pub fn new(level: i32) -> Self;

    /// Compress data using ZSTD
    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Decompress data using ZSTD
    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Compress multiple blocks in batch
    pub fn compress_blocks(&self, blocks: &[Vec<u8>]) -> Result<Vec<(usize, Vec<u8>)>>;
}
```

## Extension Points

### Custom Change Tracker

Implement the `ChangeTracker` trait to support additional change detection backends:

```rust
use helix::tracker::{ChangeTracker, Checkpoint};

struct CustomTracker;

impl ChangeTracker for CustomTracker {
    fn get_changed_blocks(&self, since: Option<Checkpoint>) -> Result<Vec<u64>> {
        // Implementation
    }

    fn create_checkpoint(&self) -> Result<Checkpoint> {
        // Implementation
    }

    fn reset_tracking(&self) -> Result<()> {
        // Implementation
    }

    fn get_current_checkpoint(&self) -> Result<Option<Checkpoint>> {
        // Implementation
    }
}
```

### Custom Storage Backend

Extend the repository layer by implementing custom block storage:

```rust
// Extend Repository or implement custom storage traits
```

## CLI Reference

```bash
# Initialize a new backup repository
helix init <path> [--key <file>] [--compression-level <1-22>]

# Full backup
helix full <source> --dest <path> [--label <name>] [--block-size <bytes>]

# Incremental backup
helix incremental <source> --dest <path> [--label <name>] [--block-size <bytes>]

# Restore from backup
helix restore <source> <target> [--point <latest|id>]

# List backups
helix list <path> [--json]

# Check repository integrity
helix check <path> [--repair]

# Configuration
helix config show
helix config validate <path>
```
