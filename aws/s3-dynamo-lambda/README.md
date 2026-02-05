# S3-DynamoDB Lambda Function

A Rust AWS Lambda function that:
1. Uploads a local CSV file to Amazon S3
2. Queries an item from DynamoDB
3. Saves combined results to S3

## Prerequisites

- Rust toolchain (1.70+)
- [cargo-lambda](https://www.cargo-lambda.info/) installed
- AWS CLI v2 configured with appropriate credentials
- AWS account with permissions for Lambda, S3, and DynamoDB

### Install cargo-lambda

```bash
# Using pip
pip install cargo-lambda

# Or using Homebrew (macOS)
brew tap cargo-lambda/cargo-lambda
brew install cargo-lambda
```

## Project Structure

```
s3-dynamo-lambda/
├── Cargo.toml
├── Makefile
├── README.md
├── src/
│   └── main.rs
└── events/
    ├── test-event.json
    └── composite-key-event.json
```

## Configuration

Edit the variables at the top of `Makefile`:

```makefile
FUNCTION_NAME := s3-dynamo-lambda
REGION := eu-west-1
TEST_BUCKET := your-test-bucket
TEST_TABLE := your-test-table
```

## IAM Permissions

Your Lambda execution role needs these permissions:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:PutObject",
        "s3:GetObject"
      ],
      "Resource": "arn:aws:s3:::your-bucket/*"
    },
    {
      "Effect": "Allow",
      "Action": [
        "dynamodb:GetItem"
      ],
      "Resource": "arn:aws:dynamodb:*:*:table/your-table"
    },
    {
      "Effect": "Allow",
      "Action": [
        "logs:CreateLogGroup",
        "logs:CreateLogStream",
        "logs:PutLogEvents"
      ],
      "Resource": "*"
    }
  ]
}
```

## Usage

### Build

```bash
# Debug build
make build

# Release build (ARM64 for Graviton)
make release
```

### Local Development

```bash
# Start local Lambda runtime
make run

# In another terminal, invoke locally
make invoke-local
```

### Deploy

```bash
# Deploy to AWS
make deploy

# Or deploy with specific IAM role
AWS_ACCOUNT_ID=123456789012 LAMBDA_ROLE_NAME=my-role make deploy-with-role
```

### Invoke

```bash
# Invoke deployed Lambda
make invoke

# View CloudWatch logs
make logs
```

## Request Format

### Simple Key (Partition Key Only)

```json
{
  "csv_file_path": "/tmp/data.csv",
  "s3_bucket": "my-bucket",
  "s3_csv_key": "uploads/data.csv",
  "s3_results_key": "results/output.json",
  "dynamo_table": "my-table",
  "partition_key_name": "pk",
  "partition_key_value": "user123"
}
```

### Composite Key (Partition + Sort Key)

```json
{
  "csv_file_path": "/mnt/efs/data.csv",
  "s3_bucket": "my-bucket",
  "s3_csv_key": "uploads/data.csv",
  "s3_results_key": "results/output.json",
  "dynamo_table": "orders",
  "partition_key_name": "user_id",
  "partition_key_value": "user123",
  "sort_key_name": "order_id",
  "sort_key_value": "order456"
}
```

## Response Format

```json
{
  "request_id": "abc-123",
  "success": true,
  "message": "CSV uploaded, DynamoDB queried, results saved to S3",
  "results_s3_location": "s3://my-bucket/results/output.json",
  "details": {
    "csv_upload": {
      "bucket": "my-bucket",
      "key": "uploads/data.csv",
      "size_bytes": 1024,
      "success": true
    },
    "dynamo_query": {
      "table": "my-table",
      "key_queried": "pk=user123",
      "item_found": true,
      "item": {
        "attributes": {
          "pk": "user123",
          "name": "John Doe",
          "email": "john@example.com"
        }
      }
    },
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

## Testing

```bash
# Run unit tests
make test

# Run with output
make test-verbose
```

## AWS Resource Setup

Helper commands to create test resources:

```bash
# Create S3 bucket
make create-bucket

# Create DynamoDB table
make create-table

# Add test item
make put-test-item
```

## Using with EFS

If your Lambda has EFS mounted (like your existing efs-lister), you can read CSV files directly from `/mnt/efs/`:

```json
{
  "csv_file_path": "/mnt/efs/exports/daily-report.csv",
  ...
}
```

## Architecture

```
┌──────────────┐     ┌─────────────────┐     ┌──────────────┐
│   Trigger    │────▶│  Lambda (Rust)  │────▶│     S3       │
│  (API/Event) │     │                 │     │ (CSV Upload) │
└──────────────┘     │  1. Read CSV    │     └──────────────┘
                     │  2. Upload S3   │
                     │  3. Query Dynamo│     ┌──────────────┐
                     │  4. Save Results│────▶│   DynamoDB   │
                     └─────────────────┘     │   (Query)    │
                            │                └──────────────┘
                            │
                            ▼
                     ┌──────────────┐
                     │     S3       │
                     │  (Results)   │
                     └──────────────┘
```

## Error Handling

The Lambda handles errors gracefully and returns structured error responses:

- **File read errors**: Returns details about missing/inaccessible files
- **S3 upload failures**: Includes AWS error messages
- **DynamoDB query failures**: Reports table/key issues
- **Partial failures**: Operations are independent; CSV upload failure doesn't prevent DynamoDB query

## License

MIT
