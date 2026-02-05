# S3-DynamoDB Lambda Makefile
# ============================
# Configure these variables for your environment

FUNCTION_NAME := s3-dynamo-lambda
REGION := eu-west-1
ARCH := arm64

# S3 and DynamoDB settings for testing
# Update these with your own unique names
TEST_BUCKET := $(USER)-lambda-test-$(REGION)
TEST_TABLE := $(USER)-lambda-test-table
TEST_PK_NAME := pk
TEST_PK_VALUE := test-key

# ============================================================================
# Development
# ============================================================================

format:
	cargo fmt --quiet

lint:
	cargo clippy --quiet -- -W clippy::pedantic

check: format lint
	cargo check

# Local development with cargo-lambda watch
run:
	cargo lambda watch

# ============================================================================
# Build
# ============================================================================

build:
	cargo lambda build

release:
	cargo lambda build --release --$(ARCH)

# ============================================================================
# Deploy
# ============================================================================

deploy: release
	cargo lambda deploy $(FUNCTION_NAME) --region $(REGION)

# Deploy with specific IAM role (required for S3/DynamoDB access)
deploy-with-role: release
	cargo lambda deploy $(FUNCTION_NAME) \
		--region $(REGION) \
		--iam-role arn:aws:iam::$(AWS_ACCOUNT_ID):role/$(LAMBDA_ROLE_NAME)

# ============================================================================
# Invoke
# ============================================================================

# Invoke locally with cargo-lambda
invoke-local:
	cargo lambda invoke --data-file events/test-event.json

# Invoke deployed Lambda via AWS CLI (requires file to exist in Lambda environment)
# NOTE: /tmp/test-data.csv must exist IN THE LAMBDA, not locally
invoke:
	aws lambda invoke \
		--function-name $(FUNCTION_NAME) \
		--payload '{"csv_file_path":"/tmp/test-data.csv","s3_bucket":"$(TEST_BUCKET)","s3_csv_key":"uploads/test-data.csv","s3_results_key":"results/output.json","dynamo_table":"$(TEST_TABLE)","partition_key_name":"$(TEST_PK_NAME)","partition_key_value":"$(TEST_PK_VALUE)"}' \
		--cli-binary-format raw-in-base64-out \
		--region $(REGION) \
		--output json \
		response.json
	@echo "\n=== Response ==="
	@cat response.json | jq .

# Invoke with auto-created test file (for testing without EFS)
invoke-test:
	aws lambda invoke \
		--function-name $(FUNCTION_NAME) \
		--payload '{"csv_file_path":"/tmp/test-data.csv","s3_bucket":"$(TEST_BUCKET)","s3_csv_key":"uploads/test-data.csv","s3_results_key":"results/output.json","dynamo_table":"$(TEST_TABLE)","partition_key_name":"$(TEST_PK_NAME)","partition_key_value":"$(TEST_PK_VALUE)","create_test_file":true}' \
		--cli-binary-format raw-in-base64-out \
		--region $(REGION) \
		--output json \
		response.json
	@echo "\n=== Response ==="
	@cat response.json | jq .

# Invoke using EFS-mounted file (recommended for production)
invoke-efs:
	aws lambda invoke \
		--function-name $(FUNCTION_NAME) \
		--payload '{"csv_file_path":"/mnt/efs/test-data.csv","s3_bucket":"$(TEST_BUCKET)","s3_csv_key":"uploads/test-data.csv","s3_results_key":"results/output.json","dynamo_table":"$(TEST_TABLE)","partition_key_name":"$(TEST_PK_NAME)","partition_key_value":"$(TEST_PK_VALUE)"}' \
		--cli-binary-format raw-in-base64-out \
		--region $(REGION) \
		--output json \
		response.json
	@echo "\n=== Response ==="
	@cat response.json | jq .

# Invoke with sort key
invoke-composite:
	aws lambda invoke \
		--function-name $(FUNCTION_NAME) \
		--payload '{"csv_file_path":"/mnt/efs/data.csv","s3_bucket":"$(TEST_BUCKET)","s3_csv_key":"uploads/data.csv","s3_results_key":"results/output.json","dynamo_table":"$(TEST_TABLE)","partition_key_name":"pk","partition_key_value":"user123","sort_key_name":"sk","sort_key_value":"order456"}' \
		--cli-binary-format raw-in-base64-out \
		--region $(REGION) \
		--output json \
		response.json
	@cat response.json | jq .

# Invoke using cargo-lambda remote
invoke-remote:
	cargo lambda invoke --remote \
		--data-file events/test-event.json \
		--output-format json \
		$(FUNCTION_NAME)

# ============================================================================
# Testing
# ============================================================================

test:
	cargo test

test-verbose:
	cargo test -- --nocapture

# ============================================================================
# AWS Resource Setup
# ============================================================================

# Create all test resources (bucket, table, test item, IAM permissions)
setup: create-bucket create-table put-test-item setup-iam
	@echo "✅ All test resources created"

# Setup IAM permissions for Lambda to access S3 and DynamoDB
setup-iam:
	@echo "Configuring IAM permissions for Lambda..."
	@ROLE_ARN=$$(aws lambda get-function-configuration \
		--function-name $(FUNCTION_NAME) \
		--region $(REGION) \
		--query 'Role' --output text 2>/dev/null) && \
	ROLE_NAME=$$(echo $$ROLE_ARN | cut -d'/' -f2) && \
	echo "Role: $$ROLE_NAME" && \
	aws iam attach-role-policy \
		--role-name $$ROLE_NAME \
		--policy-arn arn:aws:iam::aws:policy/AmazonS3FullAccess 2>/dev/null || true && \
	aws iam attach-role-policy \
		--role-name $$ROLE_NAME \
		--policy-arn arn:aws:iam::aws:policy/AmazonDynamoDBReadOnlyAccess 2>/dev/null || true && \
	echo "✅ IAM policies attached (waiting for propagation...)" && \
	sleep 5

# Create S3 bucket (ignores error if already exists)
create-bucket:
	@echo "Creating S3 bucket $(TEST_BUCKET)..."
	@aws s3 mb s3://$(TEST_BUCKET) --region $(REGION) 2>/dev/null || \
		echo "Bucket $(TEST_BUCKET) already exists or cannot be created"

# Create DynamoDB table (ignores error if already exists)
create-table:
	@echo "Creating DynamoDB table $(TEST_TABLE)..."
	@aws dynamodb create-table \
		--table-name $(TEST_TABLE) \
		--attribute-definitions AttributeName=$(TEST_PK_NAME),AttributeType=S \
		--key-schema AttributeName=$(TEST_PK_NAME),KeyType=HASH \
		--billing-mode PAY_PER_REQUEST \
		--region $(REGION) 2>/dev/null || \
		echo "Table $(TEST_TABLE) already exists or cannot be created"
	@echo "Waiting for table to be active..."
	@aws dynamodb wait table-exists --table-name $(TEST_TABLE) --region $(REGION) 2>/dev/null || true

# Add test item to DynamoDB
put-test-item:
	@echo "Adding test item to $(TEST_TABLE)..."
	@aws dynamodb put-item \
		--table-name $(TEST_TABLE) \
		--item '{"$(TEST_PK_NAME)": {"S": "$(TEST_PK_VALUE)"}, "name": {"S": "Test Item"}, "value": {"N": "42"}, "created": {"S": "$(shell date -Iseconds)"}}' \
		--region $(REGION)
	@echo "✅ Test item added"

# Check if resources exist
check-resources:
	@echo "=== S3 Bucket ==="
	@aws s3 ls s3://$(TEST_BUCKET) --region $(REGION) 2>/dev/null && echo "✅ Bucket exists" || echo "❌ Bucket not found"
	@echo "\n=== DynamoDB Table ==="
	@aws dynamodb describe-table --table-name $(TEST_TABLE) --region $(REGION) --query 'Table.TableStatus' --output text 2>/dev/null || echo "❌ Table not found"
	@echo "\n=== Test Item ==="
	@aws dynamodb get-item --table-name $(TEST_TABLE) --key '{"$(TEST_PK_NAME)": {"S": "$(TEST_PK_VALUE)"}}' --region $(REGION) 2>/dev/null | jq -r '.Item // "❌ Item not found"'

# View CloudWatch logs
logs:
	aws logs tail /aws/lambda/$(FUNCTION_NAME) --follow --region $(REGION)

# View recent logs (last 10 minutes)
logs-recent:
	aws logs tail /aws/lambda/$(FUNCTION_NAME) --since 10m --region $(REGION)

# ============================================================================
# Cleanup
# ============================================================================

clean:
	cargo clean
	rm -f response.json

# ============================================================================
# Full workflow
# ============================================================================

all: format lint test build

redeploy: release
	cargo lambda deploy $(FUNCTION_NAME) --region $(REGION)
	@echo "Deployed $(FUNCTION_NAME) to $(REGION)"

.PHONY: format lint check run build release deploy deploy-with-role \
        invoke invoke-test invoke-efs invoke-composite invoke-remote \
        test test-verbose setup setup-iam create-bucket create-table put-test-item \
        check-resources logs logs-recent clean all redeploy