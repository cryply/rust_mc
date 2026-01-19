.PHONY: setup format lint build release deploy test clean invoke all

# Load environment variables from .env if it exists
ifneq (,$(wildcard ./.env))
    include .env
    export
endif

# Project name
PROJECT_NAME = s3-dynamo-sync

# -------------------------------------------------------------------
# Development
# -------------------------------------------------------------------
format:
	cargo fmt --quiet

lint:
	cargo clippy --quiet

test:
	cargo test

run:
	cargo lambda watch

# -------------------------------------------------------------------
# AWS Infrastructure Setup
# -------------------------------------------------------------------
setup:
	@echo "Setting up AWS resources..."
	@chmod +x setup.sh
	@./setup.sh

# -------------------------------------------------------------------
# Build & Deploy
# -------------------------------------------------------------------
build:
	cargo lambda build

release:
	cargo lambda build --release --arm64

deploy: release
	@echo "Deploying Lambda function..."
	cargo lambda deploy $(PROJECT_NAME) \
		--region $(AWS_REGION) \
		--iam-role $(LAMBDA_ROLE_ARN) \
		--env-var DYNAMODB_TABLE_NAME=$(DYNAMODB_TABLE_NAME)

# Configure S3 to trigger the Lambda (run after deploy)
setup-trigger:
	@echo "Setting up S3 trigger..."
	@ACCOUNT_ID=$$(aws sts get-caller-identity --query Account --output text); \
	LAMBDA_ARN="arn:aws:lambda:$(AWS_REGION):$$ACCOUNT_ID:function:$(PROJECT_NAME)"; \
	echo "Lambda ARN: $$LAMBDA_ARN"; \
	aws lambda add-permission \
		--function-name $(PROJECT_NAME) \
		--statement-id s3-trigger-permission \
		--action lambda:InvokeFunction \
		--principal s3.amazonaws.com \
		--source-arn arn:aws:s3:::$(S3_BUCKET_NAME) \
		--source-account $$ACCOUNT_ID 2>/dev/null || echo "Permission may already exist"; \
	echo "{\"LambdaFunctionConfigurations\":[{\"LambdaFunctionArn\":\"$$LAMBDA_ARN\",\"Events\":[\"s3:ObjectCreated:*\",\"s3:ObjectRemoved:*\"]}]}" > /tmp/s3-notification.json; \
	aws s3api put-bucket-notification-configuration \
		--bucket $(S3_BUCKET_NAME) \
		--notification-configuration file:///tmp/s3-notification.json
	@echo "S3 trigger configured!"

# Full deployment (build + deploy + trigger)
full-deploy: deploy setup-trigger
	@echo "Full deployment complete!"

# -------------------------------------------------------------------
# Redeploy (quick rebuild and deploy)
# -------------------------------------------------------------------
redeploy:
	cargo lambda build --release --arm64 && \
	cargo lambda deploy $(PROJECT_NAME) \
		--region $(AWS_REGION) \
		--env-var DYNAMODB_TABLE_NAME=$(DYNAMODB_TABLE_NAME)

# -------------------------------------------------------------------
# Testing & Invocation
# -------------------------------------------------------------------

# Test with a simulated S3 event
invoke-local:
	cargo lambda invoke --data-file test-event.json

# Invoke the deployed Lambda with a test event
invoke:
	aws lambda invoke \
		--function-name $(PROJECT_NAME) \
		--payload file://test-event.json \
		--cli-binary-format raw-in-base64-out \
		--region $(AWS_REGION) \
		--output json \
		response.json
	@cat response.json | jq .

# Test by uploading a file to S3
test-upload:
	@echo "test file content - $$(date)" > /tmp/test-file.txt
	aws s3 cp /tmp/test-file.txt s3://$(S3_BUCKET_NAME)/test-file.txt
	@echo "File uploaded! Check DynamoDB in a few seconds..."
	@sleep 3
	@echo "Checking DynamoDB..."
	aws dynamodb get-item \
		--table-name $(DYNAMODB_TABLE_NAME) \
		--key '{"pk": {"S": "BUCKET#$(S3_BUCKET_NAME)"}, "sk": {"S": "KEY#test-file.txt"}}' | jq .

# View all items in DynamoDB
scan-table:
	aws dynamodb scan --table-name $(DYNAMODB_TABLE_NAME) | jq .

# View Lambda logs
logs:
	aws logs tail /aws/lambda/$(PROJECT_NAME) --follow

# -------------------------------------------------------------------
# Cleanup
# -------------------------------------------------------------------
clean:
	cargo clean
	rm -f response.json

# Full cleanup - remove AWS resources (BE CAREFUL!)
destroy:
	@echo "WARNING: This will delete all AWS resources!"
	@read -p "Are you sure? (yes/no): " confirm && [ "$$confirm" = "yes" ] || exit 1
	@echo "Removing S3 trigger..."
	-aws s3api put-bucket-notification-configuration \
		--bucket $(S3_BUCKET_NAME) \
		--notification-configuration '{}'
	@echo "Emptying S3 bucket..."
	-aws s3 rm s3://$(S3_BUCKET_NAME) --recursive
	@echo "Deleting S3 bucket..."
	-aws s3api delete-bucket --bucket $(S3_BUCKET_NAME)
	@echo "Deleting DynamoDB table..."
	-aws dynamodb delete-table --table-name $(DYNAMODB_TABLE_NAME)
	@echo "Deleting Lambda function..."
	-aws lambda delete-function --function-name $(PROJECT_NAME)
	@echo "Deleting IAM role policies..."
	-aws iam delete-role-policy --role-name $(PROJECT_NAME)-lambda-role --policy-name $(PROJECT_NAME)-policy
	-aws iam detach-role-policy --role-name $(PROJECT_NAME)-lambda-role \
		--policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole
	@echo "Deleting IAM role..."
	-aws iam delete-role --role-name $(PROJECT_NAME)-lambda-role
	@echo "Cleanup complete!"

# -------------------------------------------------------------------
# Development workflow
# -------------------------------------------------------------------
all: format lint test build