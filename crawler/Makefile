.PHONY: format lint build release run test clean help

# Default bucket and region for S3 commands
BUCKET ?= ur-crawler-bucket
REGION ?= us-west-1
PREFIX ?= crawled/
WORKERS ?= 5
URL ?= https://sabrinajewson.org/rust-nomicon/atomics/atomics.html

format:
	cargo fmt --quiet

lint:
	cargo clippy --quiet

build:
	cargo build

release:
	cargo build --release

run:
	cargo run

run-local:
	cargo run -- --storage local --workers $(WORKERS) --url $(URL)

run-s3:
	AWS_REGION=$(REGION) cargo run -- \
		--storage s3 \
		--bucket $(BUCKET) \
		--prefix $(PREFIX) \
		--workers $(WORKERS) \
		--url $(URL)

run-s3-release:
	AWS_REGION=$(REGION) cargo run --release -- \
		--storage s3 \
		--bucket $(BUCKET) \
		--prefix $(PREFIX) \
		--workers $(WORKERS) \
		--url $(URL)

test:
	cargo test

clean:
	cargo clean
	rm -rf ../data/*.html ../data/*.css ../data/*.js

# S3 utilities
s3-list:
	aws s3 ls s3://$(BUCKET)/$(PREFIX) --region $(REGION)

s3-clear:
	aws s3 rm s3://$(BUCKET)/$(PREFIX) --recursive --region $(REGION)

s3-create-bucket:
	aws s3 mb s3://$(BUCKET) --region $(REGION)

help:
	@echo "Crawler Makefile"
	@echo ""
	@echo "Usage: make [target] [VARIABLE=value]"
	@echo ""
	@echo "Targets:"
	@echo "  format          Format code with rustfmt"
	@echo "  lint            Run clippy linter"
	@echo "  build           Build debug binary"
	@echo "  release         Build release binary"
	@echo "  run             Run with default settings"
	@echo "  run-local       Run with local storage"
	@echo "  run-s3          Run with S3 storage"
	@echo "  run-s3-release  Run release build with S3 storage"
	@echo "  test            Run tests"
	@echo "  clean           Clean build artifacts and data"
	@echo "  s3-list         List objects in S3 bucket"
	@echo "  s3-clear        Delete all objects in S3 prefix"
	@echo "  s3-create-bucket Create S3 bucket"
	@echo "  help            Show this help"
	@echo ""
	@echo "Variables:"
	@echo "  BUCKET=$(BUCKET)"
	@echo "  REGION=$(REGION)"
	@echo "  PREFIX=$(PREFIX)"
	@echo "  WORKERS=$(WORKERS)"
	@echo "  URL=$(URL)"
	@echo ""
	@echo "Examples:"
	@echo "  make run-s3 BUCKET=my-bucket URL=https://example.com"
	@echo "  make run-local WORKERS=10"
	@echo "  make s3-list BUCKET=my-bucket"

all: format lint test build
