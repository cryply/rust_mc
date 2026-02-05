use lambda_runtime::{run, service_fn, Error};
use s3_dynamo_lambda::function_handler;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    info!("Starting S3-DynamoDB Lambda function");
    run(service_fn(function_handler)).await
}
