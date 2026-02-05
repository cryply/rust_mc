# S3-DynamoDB Lambda Makefile
# ============================
# Configure these variables for your environment

FUNCTION_NAME := s3-dynamo-lambda
REGION := eu-west-1
ARCH := arm64

# S3 and DynamoDB settings for testing
TEST_BUCKET := your-test-bucket
TEST_TABLE := your-test-table
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

# Invoke deployed Lambda via AWS CLI
invoke:
	@echo "Creating test CSV file..."
	@echo "id,name,value" > /tmp/test-data.csv
	@echo "1,item1,100" >> /tmp/test-data.csv
	@echo "2,item2,200" >> /tmp/test-data.csv
	aws lambda invoke \
		--function-name $(FUNCTION_NAME) \
		--payload '{ \
			"csv_file_path": "/tmp/test-data.csv", \
			"s3_bucket": "$(TEST_BUCKET)", \
			"s3_csv_key": "uploads/test-data.csv", \
			"s3_results_key": "results/output.json", \
			"dynamo_table": "$(TEST_TABLE)", \
			"partition_key_name": "$(TEST_PK_NAME)", \
			"partition_key_value": "$(TEST_PK_VALUE)" \
		}' \
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
		--payload '{ \
			"csv_file_path": "/mnt/efs/data.csv", \
			"s3_bucket": "$(TEST_BUCKET)", \
			"s3_csv_key": "uploads/data.csv", \
			"s3_results_key": "results/output.json", \
			"dynamo_table": "$(TEST_TABLE)", \
			"partition_key_name": "pk", \
			"partition_key_value": "user123", \
			"sort_key_name": "sk", \
			"sort_key_value": "order456" \
		}' \
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
# AWS Resource Setup (helpers)
# ============================================================================

# Create S3 bucket for testing
create-bucket:
	aws s3 mb s3://$(TEST_BUCKET) --region $(REGION)

# Create DynamoDB table for testing
create-table:
	aws dynamodb create-table \
		--table-name $(TEST_TABLE) \
		--attribute-definitions AttributeName=$(TEST_PK_NAME),AttributeType=S \
		--key-schema AttributeName=$(TEST_PK_NAME),KeyType=HASH \
		--billing-mode PAY_PER_REQUEST \
		--region $(REGION)

# Add test item to DynamoDB
put-test-item:
	aws dynamodb put-item \
		--table-name $(TEST_TABLE) \
		--item '{"$(TEST_PK_NAME)": {"S": "$(TEST_PK_VALUE)"}, "name": {"S": "Test Item"}, "value": {"N": "42"}}' \
		--region $(REGION)

# View CloudWatch logs
logs:
	aws logs tail /aws/lambda/$(FUNCTION_NAME) --follow --region $(REGION)

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
        invoke invoke-local invoke-composite invoke-remote \
        test test-verbose create-bucket create-table put-test-item \
        logs clean all redeploy
