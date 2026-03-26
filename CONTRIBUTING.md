# Contributing to Midenscan Indexer

Thank you for your interest in contributing to the Midenscan Indexer! This guide will help you set up your development environment and standard practices.

## Prerequisites

- **Rust**: Latest stable version (use [rustup](https://rustup.rs/))
- **Postgres**: Version 14 or higher
- **SQLx CLI**: Recommended for database interactions (`cargo install sqlx-cli`)

## Environment Setup

1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```
2. Update the `.env` file with your local database connection details.

## Development Workflow

We use a `Makefile` for common development tasks:

- **Build**: `make build`
- **Run**: `make run`
- **Test**: `make test`
- **Format**: `make fmt`
- **Lint**: `make lint`

Before submitting a Pull Request, please ensure:
1. The code is formatted: `cargo fmt --all`
2. Clippy is happy: `cargo clippy --all-targets --all-features -- -D warnings`
3. All tests pass: `cargo test`

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Commit your changes with descriptive commit messages
4. Push to your fork and submit a Pull Request
5. Ensure CI checks pass

## License

By contributing to this project, you agree that your contributions will be licensed under the MIT License found in the [LICENSE](LICENSE) file.
