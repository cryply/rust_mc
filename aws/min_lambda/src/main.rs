use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    name: String,
}

#[derive(Serialize)]
struct Response {
    message: String,
}

async fn handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let name = event.payload.name;
    
    Ok(Response {
        message: format!("Hello, {}!", name),
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
