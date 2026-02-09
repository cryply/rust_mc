# syntax=docker/dockerfile:1

# ============================================
# Stage 1: Build the Rust binary
# ============================================
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock* ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Touch main.rs to ensure rebuild with actual code
RUN touch src/main.rs

# Build the release binary
RUN cargo build --release

# ============================================
# Stage 2: Create minimal runtime image
# ============================================
FROM debian:bookworm-slim AS runtime

# Install CA certificates (needed for HTTPS if you add network calls later)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd --create-home --user-group etl

# Create data directory for output
RUN mkdir -p /data && chown etl:etl /data

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/etl-processor /app/etl-processor

# Set ownership
RUN chown etl:etl /app/etl-processor

# Switch to non-root user
USER etl

# Set environment variables
ENV RUST_LOG=info

# Default command
CMD ["/app/etl-processor"]
