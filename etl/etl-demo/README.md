# ETL Processor

A Rust-based ETL data processing pipeline containerized with Docker.

## Features

- Data extraction, transformation, and loading pipeline
- Value clamping (0-100 range) with logging
- CSV output generation
- Multi-stage Docker build for minimal production images
- Development container with hot reload

## Quick Start

### Local Development

```bash
# Run with cargo
make run

# Run tests
make test

# Format and lint
make check
```

### Docker

```bash
# Build and run production container
make docker-run

# View output
make show-output

# Development with hot reload
make docker-dev
```

## Project Structure

```
├── Cargo.toml           # Rust dependencies
├── Dockerfile           # Production multi-stage build
├── Dockerfile.dev       # Development with cargo-watch
├── docker-compose.yml   # Container orchestration
├── Makefile            # Build automation
├── src/
│   └── main.rs         # ETL logic
└── data/               # Mounted volume for output
```

## Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `RUST_LOG`          | `info`  | Log level (trace, debug, info, warn, error) |

## ETL Pipeline

1. **Extract**: Load raw data records
2. **Transform**: 
   - Filter records with `id=0`
   - Clamp values to 0-100 range
   - Log transformations
3. **Load**: Write cleaned data to CSV

## Docker Images

The production image uses a multi-stage build:
- **Builder stage**: `rust:1.83-slim-bookworm` (~1.5GB)
- **Runtime stage**: `debian:bookworm-slim` (~80MB)

Final image size: ~85MB
