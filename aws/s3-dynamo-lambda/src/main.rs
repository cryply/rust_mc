// src/main.rs

use lambda_runtime::{run, service_fn, Error};
use s3_dynamo_lambda::function_handler;
use log::LevelFilter; // Import logging level

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize env_logger.
    // Setting the filter to Info will show everything up to INFO level.
    // AWS Lambda often picks up logs from stdout/stderr automatically.
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        // Crucially, disable the default timestamp and target fields if the runtime adds them back.
        .format_timestamp(None)
        .format_target(false)
        .init();

    // The lambda_runtime function usually prints start/end messages automatically.
    // The START/END lines you see are likely from the runtime wrapper itself.
    // We only control what happens *inside* function_handler.

    run(service_fn(function_handler)).await
}