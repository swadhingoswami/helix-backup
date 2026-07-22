# Changelog

All notable changes to HELIX will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0] - 2024-03-15

### Added
- Full backup support on Linux and macOS
- Incremental backup with block-level change tracking
- dm-era integration for Linux change detection
- FSEvents integration for macOS change detection
- Software bitmap fallback tracker
- AES-256-GCM encryption
- ZSTD compression
- blake3 block hashing for integrity verification
- SQLite-based backup index and metadata
- JSON manifest for backup snapshots
- Command-line interface with clap
- YAML configuration with multiple sources
- Progress reporting with indicatif
- Parallel I/O with rayon
- Repository validation and repair
- Restore chain resolution for incremental restores
- Docker development environment
- CI/CD pipeline with GitHub Actions
- Cross-platform build matrix (Linux x86_64, macOS x86_64/ARM64)
- Comprehensive documentation (architecture, configuration, API, security)
- Example scripts and configuration files
- Unit and integration tests

### Security
- AES-256-GCM authenticated encryption
- blake3 integrity verification per block
- Secure key management
- File permission recommendations

## [0.1.0] - 2024-01-01

### Added
- Initial project structure
- Core module scaffolding
- Basic CLI interface
