.PHONY: build deploy invoke test clean

FUNCTION_NAME := lambda-template
REGION := eu-west-1

build:
	cargo lambda build --release --arm64

deploy: build
	cargo lambda deploy $(FUNCTION_NAME) --region $(REGION)

invoke:
	aws lambda invoke \
		--function-name $(FUNCTION_NAME) \
		--payload '{"name": "World"}' \
		--cli-binary-format raw-in-base64-out \
		--region $(REGION) \
		response.json
	@cat response.json | jq .

watch:
	cargo lambda watch

test:
	cargo test

clean:
	cargo clean
	rm -f response.json
