.PHONY: format lint run test all clean

format:
	cargo fmt --quiet

lint:
	cargo clippy --quiet -- -W clippy::pedantic

run:
	RUST_LOG=info cargo run

# Run with debug logging
run-debug:
	RUST_LOG=debug cargo run

# Run with trace logging (very verbose)
run-trace:
	RUST_LOG=trace cargo run

test:
	cargo test -- --nocapture

# Run only unit tests (no network)
test-unit:
	cargo test --lib -- --nocapture

# Run only integration tests
test-integration:
	cargo test --test integration_tests -- --nocapture

# Run tests with logging
test-verbose:
	RUST_LOG=info cargo test -- --nocapture

build:
	cargo build --release

# Check compilation without building
check:
	cargo check

# Generate documentation
docs:
	cargo doc --open

# Clean build artifacts
clean:
	cargo clean

all: format lint test run
