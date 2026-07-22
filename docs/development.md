# HELIX Development Guide

## Prerequisites

- **Rust**: 1.76 or later (install via [rustup](https://rustup.rs/))
- **Cargo**: Bundled with Rust
- **Linux**: `linux-headers`, `libsqlite3-dev`, `libfuse-dev`, `device-mapper-dev`
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)

## Quick Start

```bash
# Clone the repository
git clone https://github.com/helix/helix.git
cd helix

# Build
cargo build

# Run tests
cargo test

# Run with --help
cargo run -- --help
```

## Project Structure

```
helix/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library root
│   ├── cli/                 # Command-line interface
│   │   ├── commands.rs      # Command definitions and dispatch
│   │   └── parser.rs        # Argument parsing utilities
│   ├── config/              # Configuration management
│   │   ├── loader.rs        # Load from file/env/cli
│   │   └── validator.rs     # Configuration validation
│   ├── backup/              # Backup orchestration
│   │   ├── engine.rs        # Core backup engine
│   │   ├── full.rs          # Full backup logic
│   │   └── incremental.rs   # Incremental backup logic
│   ├── restore/             # Restore logic
│   │   ├── engine.rs        # Restore engine
│   │   ├── full.rs          # Full restore
│   │   └── incremental.rs   # Incremental restore
│   ├── tracker/             # Change detection
│   │   ├── bitmap.rs        # Software bitmap tracker
│   │   ├── sqlite_store.rs  # Persistent checkpoint store
│   │   ├── linux/           # Linux dm-era tracker
│   │   └── macos/           # macOS FSEvents tracker
│   ├── block/               # Block device operations
│   │   ├── device.rs        # Block device I/O
│   │   ├── hasher.rs        # blake3 hashing
│   │   └── mapper.rs        # File-to-block mapping
│   ├── repository/          # Backup repository
│   │   ├── layout.rs        # Repository structure
│   │   ├── manifest.rs      # JSON manifest format
│   │   └── index.rs         # SQLite index
│   ├── crypto/              # Security
│   │   ├── encryption.rs    # AES-256-GCM
│   │   └── compression.rs   # ZSTD compression
│   └── utils/               # Utilities
│       ├── errors.rs        # Error types
│       ├── logger.rs        # Logging setup
│       └── progress.rs      # Progress tracking
├── docs/                    # Documentation
├── examples/                # Example files
├── .github/workflows/       # CI/CD pipelines
└── Dockerfile               # Development container
```

## Development Workflow

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# With all features
cargo build --features "full"

# Check compilation without building
cargo check
```

### Testing

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test block::hasher::tests

# Run tests with output
cargo test -- --nocapture

# Run tests with specific filter
cargo test backup

# Run benchmarks
cargo bench
```

### Linting and Formatting

```bash
# Clippy lints
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

### Feature Flags

| Feature | Description | Default |
|---|---|---|
| `full` | All features enabled | Yes |
| `encryption` | AES-256-GCM encryption | Yes (with `full`) |
| `compression` | ZSTD compression | Yes (with `full`) |
| `progress` | Progress bar UI | Yes (with `full`) |

## Code Conventions

### Style

- Follow Rust standard formatting (`cargo fmt`)
- Use `camelCase` for type names, `snake_case` for functions/variables
- Prefer `Result<T, anyhow::Error>` for fallible functions
- Use `thiserror` for library error types

### Documentation

- All public items must have doc comments
- Include `# Examples` in doc comments where appropriate
- Use `///` for doc comments, `//!` for module-level docs

### Testing

- Unit tests in a `#[cfg(test)] mod tests` block at the bottom of each module
- Integration tests in `tests/` directory
- Use `rstest` for parameterized tests

## Building for Cross-Compilation

```bash
# Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# macOS x86_64
cargo build --release --target x86_64-apple-darwin

# macOS ARM64 (Apple Silicon)
cargo build --release --target aarch64-apple-darwin
```

## Docker Development

```bash
# Build the development container
docker compose build

# Run tests in container
docker compose run --rm helix cargo test

# Open a shell in the container
docker compose run --rm helix /bin/sh
```

## Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- full /dev/sda --dest /tmp/backups

# Trace-level logging
RUST_LOG=trace cargo run -- list /tmp/backups

# Profile with perf (Linux)
cargo build --release
perf record ./target/release/helix full /dev/sda --dest /tmp/backups
perf report
```
