# Miden Indexer

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Miden Indexer** is a blockchain data indexing service for the [Miden](https://github.com/0xPolygonMiden/miden-node) ecosystem. It probes the Miden Node for new blocks, processes them, and stores the relevant data (accounts, notes, transactions, nullifiers) in a PostgreSQL database.

This project is part of the Midenscan ecosystem, including the [Midenscan Backend](https://github.com/gateway-fm/midenscan-backend) and [Midenscan Frontend](https://github.com/gateway-fm/midenscan-frontend).

## Prerequisites

- Rust (latest stable)
- PostgreSQL 14+
- Access to a Miden Node RPC endpoint

## Quick Start

1. **Clone the repository**:
   ```bash
   git clone https://github.com/gateway-fm/midenscan-indexer.git
   cd midenscan-indexer
   ```

2. **Setup environment variables**:
   ```bash
   cp .env.example .env
   # Update .env with your PostgreSQL credentials and RPC URL
   ```

3. **Database Migration**:
   The indexer assumes the database schema exists. Please run the migrations in the [backend repository](https://github.com/gateway-fm/midenscan-backend) first.

4. **Run the Indexer**:
   ```bash
   make run
   ```

## Development

The project includes a `Makefile` with helpful targets for development:

```bash
make build    # Build in debug mode
make test     # Run all tests
make fmt      # Format the code base
make lint     # Run clippy checks
make release  # Build in release mode
```

## Monitoring

The indexer exposes several helpful endpoints:
- **Health check**: `GET /health` (standard JSON status response)
- **Prometheus metrics**: `GET /metrics`

Metrics include:
- `midenscan_indexer_last_indexed_block`: Last block number we've finished indexing
- `midenscan_indexer_latest_block`: Current known chain tip

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on setting up your environment and our development workflow.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
