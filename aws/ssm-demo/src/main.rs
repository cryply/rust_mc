use aws_sdk_ssm::Client as SsmClient;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    /// SSM parameter names to fetch (can be paths like /app/prod/db_url)
    parameters: Vec<String>,
    /// If true, decrypt SecureString parameters
    #[serde(default = "default_decrypt")]
    with_decryption: bool,
}

fn default_decrypt() -> bool {
    true
}

#[derive(Serialize)]
struct Response {
    req_id: String,
    parameters: Vec<ParameterResult>,
}

#[derive(Serialize)]
struct ParameterResult {
    name: String,
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

struct AppState {
    ssm_client: SsmClient,
}

async fn function_handler(
    state: &AppState,
    event: LambdaEvent<Request>,
) -> Result<Response, Error> {
    let req = event.payload;

    let mut results = Vec::with_capacity(req.parameters.len());

    for param_name in &req.parameters {
        let result = state
            .ssm_client
            .get_parameter()
            .name(param_name)
            .with_decryption(req.with_decryption)
            .send()
            .await;

        match result {
            Ok(output) => {
                let value = output.parameter().and_then(|p| p.value().map(String::from));
                results.push(ParameterResult {
                    name: param_name.clone(),
                    value,
                    error: None,
                });
            }
            Err(e) => {
                let error_msg = format!("{:?}", e);  // Debug формат покажет детали
                tracing::warn!(
                    parameter = %param_name, 
                    error = %error_msg,
                    "Failed to fetch parameter"
                );
                results.push(ParameterResult {
                    name: param_name.clone(),
                    value: None,
                    error: Some(error_msg),
                });
            }
        }
    }

    Ok(Response {
        req_id: event.context.request_id,
        parameters: results,
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let config = aws_config::load_from_env().await;
    let ssm_client = SsmClient::new(&config);
    let state = AppState { ssm_client };

    run(service_fn(|event| function_handler(&state, event))).await
}
