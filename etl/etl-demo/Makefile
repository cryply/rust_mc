.PHONY: format lint test run build clean docker-build docker-run docker-dev docker-clean

# ============================================
# Local Development
# ============================================

format:
	cargo fmt --quiet

lint:
	cargo clippy --quiet -- -D warnings

test:
	cargo test

run:
	RUST_LOG=info cargo run

build:
	cargo build --release

clean:
	cargo clean
	rm -rf data/*.csv

# ============================================
# Docker Commands
# ============================================

# Build production image
docker-build:
	docker compose build etl-processor

# Run ETL in production container
docker-run: docker-build
	mkdir -p data
	docker compose run --rm etl-processor

# Run development container with hot reload
docker-dev:
	mkdir -p data
	docker compose --profile dev up --build etl-dev

# Stop all containers
docker-stop:
	docker compose down

# Clean Docker resources
docker-clean:
	docker compose down --rmi local --volumes

# View logs
docker-logs:
	docker compose logs -f

# ============================================
# CI/CD Helpers
# ============================================

# Run all checks
check: format lint test

# Full CI pipeline
ci: check docker-build
	@echo "CI pipeline passed"

# ============================================
# Convenience Targets
# ============================================

all: format lint test run

# Show output after ETL run
show-output:
	@cat data/output.csv 2>/dev/null || echo "No output file found"
