# Contributing to metrics-bridge

Thank you for your interest in contributing!

## Development Setup

1. Install Rust (1.75+): https://rustup.rs/
2. Clone the repository
3. Run tests: `cargo test`
4. Run clippy: `cargo clippy --all-targets -- -D warnings`

## Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Ensure tests pass: `cargo test`
5. Ensure clippy is clean: `cargo clippy --all-targets -- -D warnings`
6. Commit with a descriptive message
7. Push and create a Pull Request

## Code Style

- Follow Rust conventions
- Run `cargo fmt` before committing
- Keep functions small and focused
- Add tests for new functionality
- Document public APIs

## Adding New Source Types

To add support for a new metric source:

1. Create a new file in `src/source/`
2. Implement the `Source` trait
3. Add the new type to `SourceType` enum in `config.rs`
4. Update `SourceRegistry::from_config()` to handle the new type
5. Add tests
6. Update documentation

## Reporting Issues

- Use GitHub Issues
- Include reproduction steps
- Include relevant logs and configuration (redact secrets!)
- Specify versions (Rust, Docker, OS)
