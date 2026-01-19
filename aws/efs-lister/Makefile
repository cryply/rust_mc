format:
	cargo fmt --quiet

lint:
	cargo clippy --quiet
run:
	cargo lambda watch 

release:
	cargo lambda build --release

deploy:
	cargo lambda deploy

redeploy:
	cargo lambda build --release --arm64 && cargo lambda deploy efs-lister --region eu-west-1

invoke: 
	aws lambda invoke \
	--function-name efs-lister \
	--payload '{"name": "EFS Test"}' \
	--cli-binary-format raw-in-base64-out \
	--region eu-west-1 \
	--output json \
	response.json
	cat response.json | jq .

invoke-remote:
	cargo lambda invoke --remote \
  		--data-ascii '{"name": "Marco"}' \
  		--output-format json \
  		efs-lister


test: 
	cargo test

all: format lint test run