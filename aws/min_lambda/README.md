# Lambda Template (Rust)

Minimal AWS Lambda function in Rust.

## Prerequisites

- [Rust](https://rustup.rs/)
- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html)
- [AWS CLI v2](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html)

```bash
# Install cargo-lambda
brew tap cargo-lambda/cargo-lambda
brew install cargo-lambda

# Configure AWS
aws configure
```

## Project Structure

```
lambda-template/
├── src/
│   └── main.rs      # Lambda handler
├── Cargo.toml       # Dependencies
├── Makefile         # Build commands
└── README.md
```

## Usage

### Local Development

```bash
make watch
```

Test locally with curl:
```bash
curl -X POST http://localhost:9000/lambda-url/lambda-template \
  -H "Content-Type: application/json" \
  -d '{"name": "World"}'
```

### Deploy

```bash
make deploy
```

### Invoke

```bash
make invoke
```

Expected response:
```json
{
  "message": "Hello, World!"
}
```

## Request/Response

**Request:**
```json
{
  "name": "string"
}
```

**Response:**
```json
{
  "message": "Hello, {name}!"
}
```

## Configuration

Edit `Makefile` to change:

| Variable | Default | Description |
|----------|---------|-------------|
| `FUNCTION_NAME` | `lambda-template` | Lambda function name |
| `REGION` | `eu-west-1` | AWS region |

## Commands

| Command | Description |
|---------|-------------|
| `make build` | Build for ARM64 |
| `make deploy` | Build and deploy |
| `make invoke` | Invoke remote function |
| `make watch` | Local dev server |
| `make test` | Run tests |
| `make clean` | Remove artifacts |

## Adding Dependencies

Common additions for AWS Lambda projects:

```toml
# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }

# AWS SDK
aws-config = "1.0"
aws-sdk-s3 = "1.0"
aws-sdk-dynamodb = "1.0"

# S3 events
aws_lambda_events = "0.15"

# Time
chrono = { version = "0.4", features = ["serde"] }
```

## License

MIT
