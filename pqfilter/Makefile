.PHONY: all build release test clean fmt lint check install help demo

# Default target
all: fmt lint test build

# Build debug version
build:
	cargo build

# Build optimized release version
release:
	cargo build --release

# Run all tests
test:
	cargo test

# Run tests with output
test-verbose:
	cargo test -- --nocapture

# Format code
fmt:
	cargo fmt

# Check formatting without modifying
fmt-check:
	cargo fmt --check

# Run clippy linter
lint:
	cargo clippy -- -D warnings

# Run all checks (formatting + linting + tests)
check: fmt-check lint test

# Clean build artifacts
clean:
	cargo clean
	rm -f *.parquet *.csv *.json response.json

# Install to ~/.cargo/bin
install: release
	cargo install --path .

# Uninstall
uninstall:
	cargo uninstall pqfilter

# Generate sample parquet file for testing
sample:
	@echo "Creating sample parquet file..."
	@test -d .venv || python3 -m venv .venv
	@.venv/bin/pip install -q polars
	@.venv/bin/python scripts/generate_sample.py

# Demo commands (requires sample.parquet)
demo: sample release
	@echo "\n=== Dataset Info ==="
	./target/release/pqfilter info -i sample.parquet --stats --head 5
	
	@echo "\n=== Filter: age > 50 ==="
	./target/release/pqfilter filter -i sample.parquet -o filtered.parquet -f "age:gt:50"
	./target/release/pqfilter info -i filtered.parquet --head 5
	
	@echo "\n=== Filter: active users in Engineering ==="
	./target/release/pqfilter filter -i sample.parquet -o active_eng.parquet \
		-f "status:eq:active" -f "department:eq:Engineering"
	./target/release/pqfilter info -i active_eng.parquet --head 5
	
	@echo "\n=== Select columns ==="
	./target/release/pqfilter select -i sample.parquet -o selected.csv -c name,age,score
	head -10 selected.csv
	
	@echo "\n=== Transform: uppercase names ==="
	./target/release/pqfilter transform -i sample.parquet -o transformed.parquet \
		-t "name:uppercase" -t "score:round:0"
	./target/release/pqfilter info -i transformed.parquet --head 5
	
	@echo "\n=== Aggregate by department ==="
	./target/release/pqfilter aggregate -i sample.parquet -o agg.json \
		-g department -a "age:mean" -a "score:sum" -a "id:count"
	cat agg.json | .venv/bin/python -m json.tool 2>/dev/null || cat agg.json
	
	@echo "\n=== Sort by score descending ==="
	./target/release/pqfilter sort -i sample.parquet -o sorted.parquet --by="-score"
	./target/release/pqfilter info -i sorted.parquet --head 5
	
	@echo "\n=== Sample 10 rows ==="
	./target/release/pqfilter sample -i sample.parquet -o sampled.parquet -n 10 --seed 42
	./target/release/pqfilter info -i sampled.parquet
	
	@echo "\n✅ Demo complete! Generated files: filtered.parquet, active_eng.parquet, selected.csv, transformed.parquet, agg.json, sorted.parquet, sampled.parquet"

# Show help
help:
	@echo "pqfilter - Parquet file filter and transform CLI"
	@echo ""
	@echo "Build targets:"
	@echo "  make build    - Build debug version"
	@echo "  make release  - Build optimized release"
	@echo "  make install  - Install to ~/.cargo/bin"
	@echo "  make clean    - Remove build artifacts"
	@echo ""
	@echo "Quality targets:"
	@echo "  make fmt      - Format code"
	@echo "  make lint     - Run clippy"
	@echo "  make test     - Run tests"
	@echo "  make check    - Run all checks"
	@echo ""
	@echo "Demo targets:"
	@echo "  make sample   - Generate sample.parquet"
	@echo "  make demo     - Run demo commands"
	@echo ""
	@echo "Commands (after build):"
	@echo "  pqfilter filter    - Filter rows by conditions"
	@echo "  pqfilter select    - Select/exclude columns"
	@echo "  pqfilter transform - Transform column values"
	@echo "  pqfilter sort      - Sort by columns"
	@echo "  pqfilter aggregate - Group and aggregate"
	@echo "  pqfilter info      - Show schema and stats"
	@echo "  pqfilter sample    - Sample rows"
	@echo "  pqfilter pipeline  - Run multi-step pipeline"