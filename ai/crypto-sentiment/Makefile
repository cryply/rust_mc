.PHONY: build run test check fmt lint clean release docker

build:
	cargo build

release:
	cargo build --release

run:
	cargo run -- "$(TEXT)"

test:
	cargo test

check:
	cargo check

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

lint:
	cargo clippy -- -D warnings

clean:
	cargo clean

docker:
	docker compose build

docker-run:
	docker compose run crypto-sentiment "$(TEXT)"
