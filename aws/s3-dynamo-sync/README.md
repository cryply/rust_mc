# S3-DynamoDB Sync Lambda

A Rust AWS Lambda function that automatically synchronizes S3 object metadata to DynamoDB when objects are created or deleted.

## Architecture

```
┌─────────┐    S3 Event    ┌─────────────────┐    Put/Delete    ┌──────────┐
│   S3    │ ──────────────►│  Lambda (Rust)  │ ────────────────►│ DynamoDB │
│ Bucket  │                │  s3-dynamo-sync │                  │  Table   │
└─────────┘                └─────────────────┘                  └──────────┘
```

## Prerequisites

- Rust (with `cargo-lambda` installed)
- AWS CLI v2 (authenticated)
- Environment variables set:
  - `AWS_REGION`
  - `AWS_PROFILE` (optional)

## Quick Start

### 1. Install cargo-lambda (if not already installed)

```bash
cargo install cargo-lambda
```

### 2. Setup AWS Resources

```bash
chmod +x setup.sh
./setup.sh
# or
make setup
```

This creates:
- S3 bucket with versioning enabled
- DynamoDB table (pk/sk schema, on-demand billing)
- IAM role with necessary permissions

### 3. Build & Deploy

```bash
# Source the environment variables
source .env

# Build and deploy
make full-deploy
```

### 4. Test It

```bash
# Upload a test file
make test-upload

# View all synced items
make scan-table

# Watch logs
make logs
```

## DynamoDB Schema

| Attribute     | Type   | Description                        |
|---------------|--------|------------------------------------|
| pk            | String | `BUCKET#<bucket-name>` (partition) |
| sk            | String | `KEY#<object-key>` (sort)          |
| bucket        | String | S3 bucket name                     |
| object_key    | String | S3 object key                      |
| size          | Number | Object size in bytes               |
| etag          | String | S3 ETag                            |
| content_type  | String | MIME type                          |
| last_modified | String | Last modified timestamp            |
| event_type    | String | S3 event that triggered sync       |
| synced_at     | String | When the sync occurred             |

## Makefile Commands

| Command          | Description                                    |
|------------------|------------------------------------------------|
| `make setup`     | Create AWS resources (S3, DynamoDB, IAM)       |
| `make build`     | Build the Lambda                               |
| `make release`   | Build optimized ARM64 release                  |
| `make deploy`    | Deploy Lambda to AWS                           |
| `make setup-trigger` | Configure S3 to trigger Lambda            |
| `make full-deploy` | Build + Deploy + Setup trigger               |
| `make redeploy`  | Quick rebuild and deploy                       |
| `make test-upload` | Upload test file and verify sync             |
| `make scan-table`| View all DynamoDB items                        |
| `make logs`      | Tail Lambda CloudWatch logs                    |
| `make run`       | Run locally with cargo-lambda                  |
| `make destroy`   | Delete all AWS resources (careful!)            |

## Local Development

```bash
# Run locally (simulates Lambda runtime)
make run

# In another terminal, invoke with test event
make invoke-local
```

## Cleanup

```bash
# Remove all AWS resources
make destroy
```

## Cost Considerations

- **DynamoDB**: On-demand billing - pay per request
- **Lambda**: Free tier: 1M requests/month, 400,000 GB-seconds
- **S3**: Standard storage and request pricing

For small to medium workloads, this should be nearly free!
