use lambda_runtime::{Error, LambdaEvent, run, service_fn};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    name: String,
}

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
    files: String,
}

async fn list_files() -> Result<String, Error> {
    let mut files = String::new();
    
    // Use ? to propagate errors
    let entries = std::fs::read_dir("/mnt/efs").map_err(|e| {
        // Very useful for debugging EFS issues
        eprintln!("EFS read_dir error: {e}");
        e
    })?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            files.push_str(&format!("{}\n", path.display()));
        }
    }

    if files.is_empty() {
        files = "No files found (or directory empty)".to_string();
    }

    Ok(files)
}




async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Extract some useful info from the request
    let name = event.payload.name;
    let files = list_files().await?;
    // Prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("Hello, {}!!!", name),
        files,
    };

    // Return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_function_handler() {
        let event = LambdaEvent::new(Request { name: "Test".to_string() }, lambda_runtime::Context::default());
        let result = function_handler(event).await.unwrap();
        assert_eq!(result.msg, "Hello, Test!");
    }
}