.PHONY: build run test clean init demo create list help

# Default target
help:
	@echo "PostgreSQL Encrypted Client - Available commands:"
	@echo ""
	@echo "  make build     - Build the project"
	@echo "  make test      - Run tests"
	@echo "  make init      - Initialize database schema"
	@echo "  make demo      - Run the encryption demo"
	@echo "  make list      - List all users"
	@echo "  make clean     - Clean build artifacts"
	@echo ""
	@echo "  make create USERNAME=john EMAIL=john@example.com - Create user"
	@echo "  make get ID=1                                    - Get user by ID"
	@echo "  make show-encrypted ID=1                         - Show raw encrypted data"

build:
	cargo build --release

test:
	cargo test

init:
	cargo run -- init

demo:
	cargo run -- demo

list:
	cargo run -- list

create:
	cargo run -- create --username $(USERNAME) --email $(EMAIL)

get:
	cargo run -- get --id $(ID)

show-encrypted:
	cargo run -- show-encrypted --id $(ID)

# Docker PostgreSQL for local development
db-start:
	docker run -d \
		--name pg-encrypted-demo \
		-e POSTGRES_PASSWORD=password \
		-e POSTGRES_DB=encrypted_demo \
		-p 5432:5432 \
		postgres:16

db-stop:
	docker stop pg-encrypted-demo && docker rm pg-encrypted-demo

db-psql:
	docker exec -it pg-encrypted-demo psql -U postgres -d encrypted_demo

clean:
	cargo clean
