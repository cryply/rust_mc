use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use lambda_runtime::{Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{error, info};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum LambdaError {
    #[error("S3 upload failed: {0}")]
    S3Upload(String),

    #[error("S3 download failed: {0}")]
    S3Download(String),

    #[error("DynamoDB query failed: {0}")]
    DynamoQuery(String),

    #[error("File read error: {0}")]
    FileRead(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Input event for the Lambda function
#[derive(Deserialize, Debug)]
pub struct Request {
    /// Path to local CSV file (e.g., /mnt/efs/data.csv or embedded in Lambda)
    pub csv_file_path: String,

    /// S3 bucket name for uploads
    pub s3_bucket: String,

    /// S3 key (path) for the uploaded CSV
    pub s3_csv_key: String,

    /// S3 key for the results file
    pub s3_results_key: String,

    /// `DynamoDB` table name
    pub dynamo_table: String,

    /// Partition key name in `DynamoDB`
    pub partition_key_name: String,

    /// Partition key value to query
    pub partition_key_value: String,

    /// Optional: Sort key name (if table has composite key)
    #[serde(default)]
    pub sort_key_name: Option<String>,

    /// Optional: Sort key value
    #[serde(default)]
    pub sort_key_value: Option<String>,

    /// Optional: Create a test CSV file if it doesn't exist (for testing)
    #[serde(default)]
    pub create_test_file: bool,
}

impl Request {
    /// Format the `DynamoDB` key as a string for logging/results
    pub fn format_key_string(&self) -> String {
        match (&self.sort_key_name, &self.sort_key_value) {
            (Some(sk_name), Some(sk_value)) => {
                format!(
                    "{}={}, {}={}",
                    self.partition_key_name, self.partition_key_value, sk_name, sk_value
                )
            }
            _ => format!("{}={}", self.partition_key_name, self.partition_key_value),
        }
    }
}

/// `DynamoDB` item representation for serialization
#[derive(Serialize, Debug)]
pub struct DynamoItem {
    pub attributes: HashMap<String, String>,
}

/// Combined results to save to S3
#[derive(Serialize, Debug)]
pub struct ProcessingResults {
    pub csv_upload: CsvUploadResult,
    pub dynamo_query: DynamoQueryResult,
    pub timestamp: String,
}

#[derive(Serialize, Debug)]
pub struct CsvUploadResult {
    pub bucket: String,
    pub key: String,
    pub size_bytes: u64,
    pub success: bool,
}

#[derive(Serialize, Debug)]
pub struct DynamoQueryResult {
    pub table: String,
    pub key_queried: String,
    pub item_found: bool,
    pub item: Option<DynamoItem>,
}

/// Lambda response
#[derive(Serialize, Debug)]
pub struct Response {
    pub request_id: String,
    pub success: bool,
    pub message: String,
    pub results_s3_location: String,
    pub details: Option<ProcessingResults>,
}

impl Response {
    /// Create an error response
    pub fn error(request_id: String, message: String) -> Self {
        Self {
            request_id,
            success: false,
            message,
            results_s3_location: String::new(),
            details: None,
        }
    }

    /// Create an error response with partial results
    pub fn error_with_details(
        request_id: String,
        message: String,
        details: ProcessingResults,
    ) -> Self {
        Self {
            request_id,
            success: false,
            message,
            results_s3_location: String::new(),
            details: Some(details),
        }
    }

    /// Create a success response
    pub fn success(request_id: String, results_location: String, details: ProcessingResults) -> Self {
        Self {
            request_id,
            success: true,
            message: "CSV uploaded, DynamoDB queried, results saved to S3".to_string(),
            results_s3_location: results_location,
            details: Some(details),
        }
    }
}

// ============================================================================
// AWS Client Initialization
// ============================================================================

/// Initialize AWS SDK clients with default configuration
pub async fn init_aws_clients() -> (S3Client, DynamoClient) {
    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;

    let s3_client = S3Client::new(&config);
    let dynamo_client = DynamoClient::new(&config);

    (s3_client, dynamo_client)
}

// ============================================================================
// S3 Operations
// ============================================================================

/// Create a test CSV file for testing purposes
pub async fn create_test_csv(file_path: &str) -> Result<(), LambdaError> {
    info!("Creating test CSV file at {}", file_path);

    let content = "id,name,value\n1,item1,100\n2,item2,200\n3,item3,300\n";

    tokio::fs::write(file_path, content).await.map_err(|e| {
        error!("Failed to create test file {file_path}: {e}");
        LambdaError::FileRead(format!("Failed to create test file: {e}"))
    })?;

    info!("Test CSV file created successfully");
    Ok(())
}

/// Upload a local file to S3
pub async fn upload_csv_to_s3(
    s3_client: &S3Client,
    file_path: &str,
    bucket: &str,
    key: &str,
) -> Result<u64, LambdaError> {
    info!("Uploading file {} to s3://{}/{}", file_path, bucket, key);

    // Read file contents
    let file_bytes = tokio::fs::read(file_path).await.map_err(|e| {
        error!("Failed to read file {file_path}: {e}");
        LambdaError::FileRead(format!("{file_path}: {e}"))
    })?;

    let file_size = file_bytes.len() as u64;
    info!("File size: {} bytes", file_size);

    // Create ByteStream from file contents
    let body = ByteStream::from(file_bytes);

    // Upload to S3
    s3_client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
        .content_type("text/csv")
        .send()
        .await
        .map_err(|e| {
            error!("S3 upload failed: {}", e);
            LambdaError::S3Upload(e.to_string())
        })?;

    info!("Successfully uploaded to s3://{}/{}", bucket, key);
    Ok(file_size)
}

/// Upload results JSON to S3
pub async fn save_results_to_s3(
    s3_client: &S3Client,
    results: &ProcessingResults,
    bucket: &str,
    key: &str,
) -> Result<(), LambdaError> {
    info!("Saving results to s3://{}/{}", bucket, key);

    let json = serde_json::to_string_pretty(results).map_err(|e| {
        error!("Failed to serialize results: {}", e);
        LambdaError::Serialization(e.to_string())
    })?;

    let body = ByteStream::from(json.into_bytes());

    s3_client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| {
            error!("Failed to save results to S3: {}", e);
            LambdaError::S3Upload(e.to_string())
        })?;

    info!("Results saved successfully");
    Ok(())
}

// ============================================================================
// DynamoDB Operations
// ============================================================================

/// Convert `DynamoDB` `AttributeValue` to a simple string representation
pub fn attribute_to_string(attr: &AttributeValue) -> String {
    match attr {
        AttributeValue::S(s) => s.clone(),
        AttributeValue::N(n) => n.clone(),
        AttributeValue::Bool(b) => b.to_string(),
        AttributeValue::Null(_) => "null".to_string(),
        AttributeValue::Ss(list) | AttributeValue::Ns(list) => format!("[{}]", list.join(", ")),
        AttributeValue::L(list) => {
            let items: Vec<String> = list.iter().map(attribute_to_string).collect();
            format!("[{}]", items.join(", "))
        }
        AttributeValue::M(map) => {
            let pairs: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, attribute_to_string(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        AttributeValue::B(blob) => format!("<binary:{} bytes>", blob.as_ref().len()),
        AttributeValue::Bs(blobs) => format!("<binary set:{} items>", blobs.len()),
        _ => "<unknown>".to_string(),
    }
}

/// Query a single item from `DynamoDB` using `GetItem`
pub async fn query_dynamo_item(
    dynamo_client: &DynamoClient,
    table_name: &str,
    pk_name: &str,
    pk_value: &str,
    sk_name: Option<&str>,
    sk_value: Option<&str>,
) -> Result<Option<DynamoItem>, LambdaError> {
    info!(
        "Querying DynamoDB table {} for {}={}",
        table_name, pk_name, pk_value
    );

    // Build the key
    let mut key_builder = dynamo_client
        .get_item()
        .table_name(table_name)
        .key(pk_name, AttributeValue::S(pk_value.to_string()));

    // Add sort key if provided
    if let (Some(sk_name), Some(sk_value)) = (sk_name, sk_value) {
        key_builder = key_builder.key(sk_name, AttributeValue::S(sk_value.to_string()));
    }

    let result = key_builder.send().await.map_err(|e| {
        error!("DynamoDB query failed: {}", e);
        LambdaError::DynamoQuery(e.to_string())
    })?;

    // Convert result to our DynamoItem type
    if let Some(item) = result.item {
        info!("Item found with {} attributes", item.len());
        let attributes: HashMap<String, String> = item
            .iter()
            .map(|(k, v)| (k.clone(), attribute_to_string(v)))
            .collect();

        Ok(Some(DynamoItem { attributes }))
    } else {
        info!("No item found for key {}={}", pk_name, pk_value);
        Ok(None)
    }
}

// ============================================================================
// Lambda Handler - Step Functions
// ============================================================================

/// Step 1: Upload CSV to S3 and build result
pub async fn step_upload_csv(
    s3_client: &S3Client,
    payload: &Request,
) -> Result<CsvUploadResult, String> {
    upload_csv_to_s3(
        s3_client,
        &payload.csv_file_path,
        &payload.s3_bucket,
        &payload.s3_csv_key,
    )
    .await
    .map(|size| CsvUploadResult {
        bucket: payload.s3_bucket.clone(),
        key: payload.s3_csv_key.clone(),
        size_bytes: size,
        success: true,
    })
    .map_err(|e| e.to_string())
}

/// Step 2: Query `DynamoDB` and build result
pub async fn step_query_dynamo(
    dynamo_client: &DynamoClient,
    payload: &Request,
) -> Result<DynamoQueryResult, String> {
    let item = query_dynamo_item(
        dynamo_client,
        &payload.dynamo_table,
        &payload.partition_key_name,
        &payload.partition_key_value,
        payload.sort_key_name.as_deref(),
        payload.sort_key_value.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(DynamoQueryResult {
        table: payload.dynamo_table.clone(),
        key_queried: payload.format_key_string(),
        item_found: item.is_some(),
        item,
    })
}

// ============================================================================
// Lambda Handler
// ============================================================================

pub async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let request_id = event.context.request_id.clone();
    let payload = event.payload;

    info!("Processing request: {}", request_id);
    info!("CSV file: {}", payload.csv_file_path);
    info!("S3 bucket: {}", payload.s3_bucket);
    info!("DynamoDB table: {}", payload.dynamo_table);

    // Optional: Create test file for testing purposes
    if payload.create_test_file {
        if let Err(e) = create_test_csv(&payload.csv_file_path).await {
            error!("Failed to create test file: {e}");
            return Ok(Response::error(
                request_id,
                format!("Failed to create test file: {e}"),
            ));
        }
    }

    let (s3_client, dynamo_client) = init_aws_clients().await;

    // Step 1: Upload CSV to S3
    let csv_upload_result = match step_upload_csv(&s3_client, &payload).await {
        Ok(result) => result,
        Err(e) => {
            error!("CSV upload failed: {e}");
            return Ok(Response::error(request_id, format!("CSV upload failed: {e}")));
        }
    };

    // Step 2: Query DynamoDB
    let dynamo_result = match step_query_dynamo(&dynamo_client, &payload).await {
        Ok(result) => result,
        Err(e) => {
            error!("DynamoDB query failed: {e}");
            return Ok(Response::error(
                request_id,
                format!("DynamoDB query failed: {e}"),
            ));
        }
    };

    // Step 3: Combine results and save to S3
    let processing_results = ProcessingResults {
        csv_upload: csv_upload_result,
        dynamo_query: dynamo_result,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let results_location = format!("s3://{}/{}", payload.s3_bucket, payload.s3_results_key);

    if let Err(e) = save_results_to_s3(
        &s3_client,
        &processing_results,
        &payload.s3_bucket,
        &payload.s3_results_key,
    )
    .await
    {
        error!("Failed to save results: {e}");
        return Ok(Response::error_with_details(
            request_id,
            format!("Failed to save results to S3: {e}"),
            processing_results,
        ));
    }

    info!("All operations completed successfully");
    Ok(Response::success(
        request_id,
        results_location,
        processing_results,
    ))
}
