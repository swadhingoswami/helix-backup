# HELIX Architecture Guide

## System Overview

HELIX is a block-level backup engine that operates directly on raw block devices. It reads data in fixed-size blocks, tracks which blocks change between backups, and stores only the changed blocks in incremental snapshots. This architecture makes it filesystem-agnostic — it does not need to understand ext4, NTFS, APFS, or any other filesystem structure.

### Core Design Principles

1. **Block-Level Operation** — All I/O happens at the block level, bypassing filesystem metadata
2. **Efficient Change Tracking** — Only changed blocks are stored in incremental backups
3. **Verifiable Integrity** — Every block is hashed with blake3 for integrity verification
4. **Cross-Platform Abstraction** — Platform-specific change tracking is abstracted behind a trait
5. **Defense in Depth** — Optional encryption (AES-256-GCM) and compression (ZSTD)

## Component Architecture

```mermaid
graph TB
    subgraph User[User Interface]
        CLI[CLI / clap]
        Config[Configuration Manager]
    end

    subgraph Core[Core Engine]
        Engine[BackupEngine]
        Restore[RestoreEngine]
        Tracker[ChangeTracker Trait]
    end

    subgraph Storage[Storage Layer]
        Repo[Repository Manager]
        Manifest[JSON Manifest]
        Index[SQLite Index]
    end

    subgraph Crypto[Security Layer]
        Encrypt[AES-256-GCM]
        Compress[ZSTD]
        Hasher[blake3]
    end

    subgraph Platform[Platform Layer]
        Linux[Linux: dm-era]
        MacOS[macOS: FSEvents]
        Block[BlockDevice]
    end

    CLI --> Engine
    CLI --> Restore
    Config --> Engine
    Config --> Restore

    Engine --> Tracker
    Engine --> Block
    Engine --> Repo
    Restore --> Repo
    Restore --> Block

    Tracker --> Linux
    Tracker --> MacOS

    Repo --> Manifest
    Repo --> Index
    Repo --> Encrypt
    Repo --> Compress

    Block --> Hasher
```

## Backup Flow

```mermaid
sequenceDiagram
    participant U as User
    participant CLI as CLI
    participant Engine as BackupEngine
    participant Device as BlockDevice
    participant Tracker as ChangeTracker
    participant Repo as Repository

    U->>CLI: helix full /dev/sda --dest /backups
    CLI->>Engine: run_full_backup("/dev/sda", "/backups")
    Engine->>Repo: open_or_create("/backups")
    Engine->>Device: open("/dev/sda", 4096)
    Device-->>Engine: device handle
    Engine->>Device: block_count()
    Device-->>Engine: total_blocks

    Engine->>Repo: create_snapshot("full-backup", "full")
    Repo-->>Engine: snapshot_id

    loop Each block
        Engine->>Device: read_block(n)
        Device-->>Engine: block_data
        Engine->>Engine: hash_block(data)
        Engine->>Repo: write_full_blocks(snapshot_id, blocks)
        Engine->>Repo: store_block_hashes(snapshot_id, hashes)
    end

    Engine->>Repo: finalize_snapshot(snapshot_id)
    Repo-->>Engine: OK
    Engine-->>CLI: completed
    CLI-->>U: Full backup completed
```

## Incremental Backup Flow

```mermaid
sequenceDiagram
    participant U as User
    participant CLI as CLI
    participant Engine as BackupEngine
    participant Device as BlockDevice
    participant Tracker as ChangeTracker
    participant Repo as Repository

    U->>CLI: helix incremental /dev/sda --dest /backups
    CLI->>Engine: run_incremental_backup("/dev/sda", "/backups")

    Engine->>Repo: open("/backups")
    Engine->>Tracker: create_tracker()
    Engine->>Repo: last_checkpoint()
    Repo-->>Engine: checkpoint

    Engine->>Tracker: get_changed_blocks(checkpoint)
    Tracker-->>Engine: [changed_block_numbers]

    alt No Changes
        Engine-->>CLI: No changes detected
        CLI-->>U: Completed
    else Changes Found
        Engine->>Repo: create_snapshot("inc-001", "incremental")

        loop Each changed block
            Engine->>Device: read_block(n)
            Device-->>Engine: block_data
            Engine->>Repo: write_incremental_blocks(snapshot_id)
            Engine->>Repo: store_block_hashes(snapshot_id)
        end

        Engine->>Tracker: create_checkpoint()
        Tracker-->>Engine: new_checkpoint
        Engine->>Repo: save_checkpoint(new_checkpoint)

        Engine->>Repo: finalize_snapshot(snapshot_id)
        Engine-->>CLI: completed
        CLI-->>U: Incremental backup completed
    end
```

## Restore Flow

```mermaid
sequenceDiagram
    participant U as User
    participant CLI as CLI
    participant Restore as RestoreEngine
    participant Repo as Repository
    participant Device as BlockDevice

    U->>CLI: helix restore /backups /dev/sda
    CLI->>Restore: run_restore("/backups", "/dev/sda", "latest")
    Restore->>Repo: open("/backups")
    Restore->>Repo: list_backups()
    Repo-->>Restore: [snapshots]
    Restore->>Repo: build_restore_chain(latest.id)
    Repo-->>Restore: [full-id, inc-001, inc-002, ...]

    Restore->>Device: open_for_write("/dev/sda", block_size)

    loop Each snapshot in chain
        Restore->>Repo: load_manifest(snapshot_id)
        Repo-->>Restore: manifest

        alt Full Snapshot
            loop Each block
                Restore->>Repo: read_full_block(snapshot_id, block_num)
                Repo-->>Restore: block_data
                Restore->>Device: write_block(block_num, data)
            end
        else Incremental Snapshot
            loop Each changed block
                Restore->>Repo: read_incremental_block(snapshot_id, block_num)
                Repo-->>Restore: block_data
                Restore->>Device: write_block(block_num, data)
            end
        end
    end

    Device->>Device: flush()
    Restore-->>CLI: completed
    CLI-->>U: Restore completed successfully
```

## Cross-Platform Strategy

### Linux: dm-era Change Tracking

```mermaid
graph LR
    A[Device Mapper] --> B[dm-era Target]
    B --> C[Era Metadata]
    C --> D[Changed Block Query]
    D --> E[Block Number List]

    style A fill:#4a90d9,color:#fff
    style B fill:#e74c3c,color:#fff
    style C fill:#27ae60,color:#fff
    style D fill:#f39c12,color:#fff
    style E fill:#9b59b6,color:#fff
```

On Linux, HELIX uses the Device Mapper era target (`dm-era`) for efficient block-level change tracking. The `dm-era` target maintains metadata about which blocks have changed since a given checkpoint. HELIX queries this metadata to identify changed blocks for incremental backups.

- Zero overhead for unchanged blocks
- Kernel-level change tracking
- Persistent metadata across reboots

### macOS: FSEvents Change Tracking

```mermaid
graph LR
    A[FSEvents API] --> B[File Change Events]
    B --> C[Path → Block Mapping]
    C --> D[Changed Block List]

    style A fill:#4a90d9,color:#fff
    style B fill:#e74c3c,color:#fff
    style C fill:#f39c12,color:#fff
    style D fill:#9b59b6,color:#fff
```

On macOS, HELIX uses the FSEvents API for file-level change detection. File changes are mapped to block numbers using APFS extent information. The mapping from files to blocks is maintained in an SQLite store.

- Uses OS-native file change notifications
- Maps file changes to block numbers
- Persistent checkpoint state in SQLite

### Fallback: Dirty Bitmap

For systems without hardware change tracking support, HELIX includes a software bitmap-based tracker that records all writes and compares block hashes to detect changes.

## Data Flow

### Write Path (Backup)

```
Block Device → Read Block → blake3 Hash → [Optional: Encrypt] → [Optional: Compress] → Write to Repository
```

### Read Path (Restore)

```
Repository → Read Block Data → [Optional: Decompress] → [Optional: Decrypt] → Verify blake3 Hash → Write to Device
```

## Security Model

### Encryption Architecture

```mermaid
graph TB
    subgraph Key[Key Management]
        KP[Key File]
        KM[KMS Provider]
    end

    subgraph Process[Encryption Process]
        KD[Key Derivation]
        E[AES-256-GCM]
        N[Nonce Generation]
    end

    subgraph Storage[Encrypted Storage]
        CT[Ciphertext + Nonce]
        AT[AAD / Metadata]
    end

    KP --> KD
    KM --> KD
    KD --> E
    N --> E
    E --> CT
    E --> AT
```

- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key Size**: 256 bits
- **Nonce**: 96-bit random per encryption operation
- **Integrity**: GCM provides authentication tag
- **Key Storage**: File-based or external KMS

## Performance Considerations

### Block Size Selection

| Block Size | Full Backup Speed | Incremental Granularity | Storage Overhead |
|---|---|---|---|
| 512 B | Slow | Fine | Low |
| 4 KB | Fast | Good | Medium |
| 64 KB | Very Fast | Coarse | High |
| 1 MB | Maximum | Very Coarse | Very High |

### I/O Concurrency

- Uses `rayon` for parallel block processing
- Configurable I/O concurrency limit
- Direct I/O support on Linux for raw device access
- Throttling capability for production environments

## Testing Strategy

### Unit Tests

- Every module has comprehensive unit tests
- Mock external dependencies (block devices, trackers)
- Test error handling and edge cases
- Use `rstest` for parameterized testing

### Integration Tests

- Full backup/restore cycle with temporary files
- Cross-platform testing in CI
- Repository validation tests
- Encryption/compression round-trip tests

### Performance Tests

- Benchmark with `cargo bench`
- Measure throughput for various block sizes
- Profile memory usage
- Track I/O patterns

## Deployment

### Production Requirements

- **Linux**: Linux kernel 5.4+, `dm-era` kernel module, 256 MB RAM, 1 CPU core
- **macOS**: macOS 11+, 512 MB RAM, 1 CPU core
- **Storage**: Sufficient space for backup repository (plan for 2x backup size during operations)

### Security Recommendations

1. Run with minimum required privileges
2. Store encryption keys separately from backup data
3. Enable integrity verification
4. Regular repository validation (`helix check`)
5. Off-site backup of repository metadata
