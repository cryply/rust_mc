# SSM Parameter Reader Lambda

A Rust Lambda function that securely reads parameters from AWS Systems Manager Parameter Store.

## Features

- Fetch multiple parameters in a single invocation
- Supports both `String` and `SecureString` parameter types
- Automatic decryption of SecureString parameters
- Graceful error handling per parameter (doesn't fail entire request if one param missing)

## Project Structure

```
ssm-reader/
├── Cargo.toml
├── Makefile
├── README.md
├── iam-policy.json
└── src/
    └── main.rs
```

## Request Format

```json
{
  "parameters": ["/app/prod/db_url", "/app/prod/api_key"],
  "with_decryption": true
}
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `parameters` | `string[]` | Yes | - | List of SSM parameter names |
| `with_decryption` | `bool` | No | `true` | Decrypt SecureString parameters |

## Response Format

```json
{
  "req_id": "abc-123",
  "parameters": [
    { "name": "/app/prod/db_url", "value": "postgres://..." },
    { "name": "/app/prod/api_key", "value": "decrypted-secret" }
  ]
}
```

If a parameter fails to fetch:

```json
{
  "name": "/app/prod/missing",
  "value": null,
  "error": "ParameterNotFound: ..."
}
```

## Setup

### 1. Create Test Parameters

```bash
make setup-params
```

Or manually:

```bash
# SecureString (encrypted)
aws ssm put-parameter \
  --name "/app/test/api_key" \
  --value "my-secret" \
  --type SecureString \
  --region eu-west-1

# String (plain text)
aws ssm put-parameter \
  --name "/app/test/db_url" \
  --value "postgres://localhost/db" \
  --type String \
  --region eu-west-1
```

### 2. Configure IAM Policy

Attach the policy in `iam-policy.json` to your Lambda execution role:

```bash
# Get your Lambda role name (after first deploy)
ROLE_NAME=$(aws lambda get-function --function-name ssm-reader \
  --query 'Configuration.Role' --output text | cut -d'/' -f2)

# Create and attach the policy
aws iam put-role-policy \
  --role-name $ROLE_NAME \
  --policy-name SSMReadAccess \
  --policy-document file://iam-policy.json
```

### 3. Build & Deploy

```bash
# Build for ARM64 (Graviton2 - cheaper & faster)
make build

# Deploy to AWS
make deploy
```

### 4. Test

```bash
make invoke
```

Expected output:

```json
{
  "req_id": "...",
  "parameters": [
    {
      "name": "/app/test/api_key",
      "value": "my-secret-api-key"
    },
    {
      "name": "/app/test/db_url", 
      "value": "postgres://user:pass@localhost:5432/mydb"
    }
  ]
}
```

## Local Development

```bash
# Start local Lambda runtime
make run

# In another terminal, invoke locally (requires AWS credentials)
make invoke-local
```

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Client    │────▶│   Lambda    │────▶│     SSM     │
│             │◀────│ ssm-reader  │◀────│ Parameter   │
└─────────────┘     └─────────────┘     │   Store     │
                           │            └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │     KMS     │
                    │ (decrypt)   │
                    └─────────────┘
```

## Security Best Practices

1. **Use SecureString** for sensitive values (API keys, passwords, tokens)
2. **Use path-based naming** (`/app/env/param`) for organization and IAM scoping
3. **Limit IAM permissions** to specific parameter paths (see `iam-policy.json`)
4. **Use custom KMS keys** for additional control over encryption
5. **Enable CloudTrail** to audit parameter access

## Makefile Targets

| Target | Description |
|--------|-------------|
| `make build` | Build release binary for ARM64 |
| `make deploy` | Build and deploy to AWS Lambda |
| `make run` | Start local development server |
| `make invoke` | Invoke deployed Lambda |
| `make invoke-local` | Invoke local Lambda |
| `make setup-params` | Create test parameters in SSM |
| `make cleanup-params` | Delete test parameters |
| `make format` | Format code with rustfmt |
| `make lint` | Run clippy lints |
| `make clean` | Clean build artifacts |

## Configuration

Edit the Makefile variables to customize:

```makefile
FUNCTION_NAME := ssm-reader    # Lambda function name
REGION := eu-west-1            # AWS region
ARCH := arm64                  # arm64 or x86_64
```
