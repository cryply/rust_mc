# Rust Web Crawler

A concurrent web crawler written in Rust with support for local filesystem and AWS S3 storage backends.

## Features

- Concurrent crawling with configurable worker count
- Automatic link extraction from HTML pages
- Duplicate URL detection via in-memory cache
- Content-type aware file handling (text vs binary)
- Storage backends: local filesystem or AWS S3

## Prerequisites

- Rust 2024 edition
- AWS credentials configured (for S3 storage)

## Installation

```bash
cargo build --release
```

## Usage

### Local Storage (default)

```bash
cargo run -- --url https://example.com
```

Files are saved to `../data/` directory.

### S3 Storage

```bash
cargo run -- --storage s3 --bucket my-bucket --url https://example.com
```

### CLI Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--storage` | `-s` | Storage type: `local` or `s3` | `local` |
| `--bucket` | `-b` | S3 bucket name (required for S3) | - |
| `--prefix` | `-p` | S3 key prefix | `crawled/` |
| `--url` | `-u` | Starting URL(s) to crawl | example URL |
| `--workers` | `-w` | Number of concurrent workers | `5` |

### Examples

Crawl multiple URLs with 10 workers:

```bash
cargo run -- --workers 10 --url https://site1.com --url https://site2.com
```

Save to S3 with custom prefix:

```bash
cargo run -- --storage s3 --bucket my-bucket --prefix "crawl-2025/" --url https://example.com
```

## AWS Configuration

The crawler uses the standard AWS credential chain:

1. Environment variables: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`
2. AWS credentials file: `~/.aws/credentials`
3. AWS config file: `~/.aws/config`

Set the region via environment variable:

```bash
export AWS_REGION=us-west-1
```

Or use a profile:

```bash
export AWS_PROFILE=my-profile
```

**Note:** If your `~/.aws/config` contains a `login_session` setting, you may need to remove it or export credentials directly as environment variables.

## Project Structure

```
crawler/
├── Cargo.toml
├── Makefile
├── README.md
└── src/
    └── main.rs
```

## License

MIT
