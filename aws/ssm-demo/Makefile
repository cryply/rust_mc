# Configuration
FUNCTION_NAME := ssm-reader
REGION := eu-west-1
ARCH := arm64

# Test parameters - create these in SSM first with `make setup-params`
TEST_PARAMS := ["/app/test/api_key", "/app/test/db_url"]

format:
	cargo fmt --quiet

lint:
	cargo clippy --quiet

run:
	cargo lambda watch

build:
	cargo lambda build --release --$(ARCH)

deploy: build
	cargo lambda deploy $(FUNCTION_NAME) --region $(REGION)

# Create test parameters in SSM
setup-params:
	aws ssm put-parameter \
		--name "/app/test/api_key" \
		--value "my-secret-api-key" \
		--type SecureString \
		--region $(REGION) \
		--overwrite || true
	aws ssm put-parameter \
		--name "/app/test/db_url" \
		--value "postgres://user:pass@localhost:5432/mydb" \
		--type String \
		--region $(REGION) \
		--overwrite || true

# Delete test parameters
cleanup-params:
	aws ssm delete-parameter --name "/app/test/api_key" --region $(REGION) || true
	aws ssm delete-parameter --name "/app/test/db_url" --region $(REGION) || true

attach-policy:
	$(eval ROLE_NAME := $(shell aws lambda get-function --function-name $(FUNCTION_NAME) --region $(REGION) --query 'Configuration.Role' --output text | awk -F'/' '{print $$NF}'))
	aws iam put-role-policy \
		--role-name $(ROLE_NAME) \
		--policy-name SSMReadAccess \
		--policy-document file://iam-policy.json	

invoke:
	aws lambda invoke \
		--function-name $(FUNCTION_NAME) \
		--payload '{"parameters": $(TEST_PARAMS)}' \
		--cli-binary-format raw-in-base64-out \
		--region $(REGION) \
		--output json \
		response.json
	@echo "Response:"
	@cat response.json | jq .

invoke-local:
	cargo lambda invoke \
		--data-ascii '{"parameters": ["/app/test/api_key", "/app/test/db_url"]}' \
		--output-format json

test:
	cargo test

clean:
	cargo clean
	rm -f response.json

all: format lint test build
