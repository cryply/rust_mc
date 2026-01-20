.PHONY: build run release clean fmt lint test all

# Development build
build:
	cargo build

# Run in development mode
run:
	RUST_LOG=info cargo run

# Release build (optimized)
release:
	cargo build --release

# Run release build
run-release:
	RUST_LOG=info cargo run --release

# Format code
fmt:
	cargo fmt

# Lint with clippy
lint:
	cargo clippy -- -D warnings

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Full build pipeline
all: fmt lint build

# Install dependencies (for Ubuntu 24.04)
deps:
	@echo "Installing system dependencies for Ubuntu 24.04..."
	sudo apt-get update
	sudo apt-get install -y \
		pkg-config \
		libx11-dev \
		libxkbcommon-dev \
		libwayland-dev \
		libvulkan-dev \
		vulkan-tools \
		mesa-vulkan-drivers

# Check Vulkan support
vulkan-info:
	vulkaninfo --summary || echo "Install vulkan-tools for this to work"