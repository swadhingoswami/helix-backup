# Contributing to HELIX

Thank you for your interest in contributing to HELIX! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to the [Contributor Covenant](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How to Contribute

### Reporting Bugs

1. Check the issue tracker to avoid duplicates
2. Use the bug report template
3. Include:
   - Operating system and version
   - HELIX version (`helix --version`)
   - Steps to reproduce
   - Expected vs actual behavior
   - Logs (run with `RUST_LOG=debug`)

### Suggesting Features

1. Check the issue tracker for existing feature requests
2. Use the feature request template
3. Describe the feature and its use case
4. Explain how it fits HELIX's architecture

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests: `cargo test`
5. Run lints: `cargo clippy -- -D warnings`
6. Format code: `cargo fmt`
7. Commit with a descriptive message
8. Push to your fork
9. Open a pull request

## Development Setup

See [Development Guide](docs/development.md) for detailed setup instructions.

### Quick Start

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/helix.git
cd helix

# Build
cargo build

# Run tests
cargo test

# Check lints
cargo clippy -- -D warnings
cargo fmt --check
```

## Coding Standards

### Style

- Follow Rust standard style (`cargo fmt`)
- Use meaningful variable names
- Keep functions focused and small
- Document public APIs with doc comments

### Testing

- Write tests for all new functionality
- Maintain or improve code coverage
- Use `#[cfg(test)] mod tests` for unit tests
- Include integration tests where appropriate

### Documentation

- Document all public items with `///`
- Update relevant documentation in `docs/`
- Include examples in doc comments
- Update README.md if adding features

## Pull Request Guidelines

1. **One PR per feature/bugfix** — keep changes focused
2. **Rebase on main** — keep commit history clean
3. **Update documentation** — keep docs in sync with code
4. **Add tests** — maintain coverage
5. **Pass CI** — all checks must pass

## Commit Message Format

```
<type>(<scope>): <description>

<body>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
Scopes: `cli`, `backup`, `restore`, `tracker`, `block`, `repo`, `crypto`, `config`, `docs`

Examples:
```
feat(backup): add parallel block hashing
fix(tracker): handle empty checkpoint list
docs(api): add restoration examples
```

## Review Process

1. Maintainers review the PR
2. CI must pass
3. Changes may be requested
4. PR is merged after approval

## Release Process

1. Version bump in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Tag release (`git tag v1.0.0`)
4. Push tag (`git push origin v1.0.0`)
5. CI builds and publishes release

## Questions?

Open an issue with your question or tag it with `question`.

Thank you for contributing!
