use aws_lambda_events::event::s3::S3Event;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_s3::Client as S3Client;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// Response structure for the Lambda function
#[derive(Serialize, Deserialize)]
struct SyncResponse {
    message: String,
    processed_count: usize,
    errors: Vec<String>,
}

/// Metadata we store in DynamoDB for each S3 object
#[derive(Debug, Serialize, Deserialize)]
struct S3ObjectMetadata {
    bucket: String,
    key: String,
    size: i64,
    etag: Option<String>,
    content_type: Option<String>,
    last_modified: String,
    event_type: String,
}

/// Get the DynamoDB table name from environment variable
fn get_table_name() -> String {
    std::env::var("DYNAMODB_TABLE_NAME").unwrap_or_else(|_| "s3-dynamo-sync-table".to_string())
}

/// Process an S3 event and sync metadata to DynamoDB
async fn sync_to_dynamodb(
    s3_client: &S3Client,
    dynamo_client: &DynamoClient,
    bucket: &str,
    key: &str,
    event_name: &str,
) -> Result<(), Error> {
    let table_name = get_table_name();
    
    // Check if this is a delete event
    let is_delete = event_name.contains("Remove") || event_name.contains("Delete");

    if is_delete {
        // Delete the item from DynamoDB
        info!(bucket = bucket, key = key, "Deleting item from DynamoDB");
        
        dynamo_client
            .delete_item()
            .table_name(&table_name)
            .key("pk", AttributeValue::S(format!("BUCKET#{}", bucket)))
            .key("sk", AttributeValue::S(format!("KEY#{}", key)))
            .send()
            .await?;
        
        info!("Successfully deleted item from DynamoDB");
    } else {
        // Get object metadata from S3
        info!(bucket = bucket, key = key, "Getting object metadata from S3");
        
        let head_result = s3_client
            .head_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?;

        let metadata = S3ObjectMetadata {
            bucket: bucket.to_string(),
            key: key.to_string(),
            size: head_result.content_length().unwrap_or(0),
            etag: head_result.e_tag().map(|s| s.to_string()),
            content_type: head_result.content_type().map(|s| s.to_string()),
            last_modified: head_result
                .last_modified()
                .map(|dt| dt.to_string())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            event_type: event_name.to_string(),
        };

        info!(metadata = ?metadata, "Syncing metadata to DynamoDB");

        // Write to DynamoDB
        dynamo_client
            .put_item()
            .table_name(&table_name)
            .item("pk", AttributeValue::S(format!("BUCKET#{}", bucket)))
            .item("sk", AttributeValue::S(format!("KEY#{}", key)))
            .item("bucket", AttributeValue::S(metadata.bucket))
            .item("object_key", AttributeValue::S(metadata.key))
            .item("size", AttributeValue::N(metadata.size.to_string()))
            .item(
                "etag",
                AttributeValue::S(metadata.etag.unwrap_or_default()),
            )
            .item(
                "content_type",
                AttributeValue::S(metadata.content_type.unwrap_or_default()),
            )
            .item("last_modified", AttributeValue::S(metadata.last_modified))
            .item("event_type", AttributeValue::S(metadata.event_type))
            .item(
                "synced_at",
                AttributeValue::S(chrono::Utc::now().to_rfc3339()),
            )
            .send()
            .await?;

        info!("Successfully synced metadata to DynamoDB");
    }

    Ok(())
}

/// Main Lambda handler for S3 events
async fn function_handler(
    s3_client: &S3Client,
    dynamo_client: &DynamoClient,
    event: LambdaEvent<S3Event>,
) -> Result<SyncResponse, Error> {
    let s3_event = event.payload;
    let mut processed_count = 0;
    let mut errors: Vec<String> = Vec::new();

    info!(
        record_count = s3_event.records.len(),
        "Processing S3 event"
    );

    for record in s3_event.records {
        let bucket = record
            .s3
            .bucket
            .name
            .as_deref()
            .unwrap_or("unknown");
        
        let key = record
            .s3
            .object
            .key
            .as_deref()
            .unwrap_or("unknown");
        
        let event_name = record
            .event_name
            .as_deref()
            .unwrap_or("unknown");

        info!(
            bucket = bucket,
            key = key,
            event = event_name,
            "Processing record"
        );

        match sync_to_dynamodb(s3_client, dynamo_client, bucket, key, event_name).await {
            Ok(_) => {
                processed_count += 1;
            }
            Err(e) => {
                let error_msg = format!("Failed to process {}/{}: {}", bucket, key, e);
                error!("{}", error_msg);
                errors.push(error_msg);
            }
        }
    }

    Ok(SyncResponse {
        message: format!(
            "Processed {} objects with {} errors",
            processed_count,
            errors.len()
        ),
        processed_count,
        errors,
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time() // CloudWatch adds timestamps
        .json() // Use JSON format for better CloudWatch parsing
        .init();

    info!("Initializing S3-DynamoDB Sync Lambda");

    // Initialize AWS SDK clients
    let config = aws_config::load_from_env().await;
    let s3_client = S3Client::new(&config);
    let dynamo_client = DynamoClient::new(&config);

    info!(
        table_name = %get_table_name(),
        "Lambda initialized"
    );

    // Run the Lambda runtime
    run(service_fn(|event: LambdaEvent<S3Event>| async {
        function_handler(&s3_client, &dynamo_client, event).await
    }))
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_lambda_events::s3::{S3Bucket, S3Entity, S3EventRecord, S3Object};

    #[allow(dead_code)]
    fn create_test_event(bucket: &str, key: &str, event_name: &str) -> S3Event {
        S3Event {
            records: vec![S3EventRecord {
                event_name: Some(event_name.to_string()),
                s3: S3Entity {
                    bucket: S3Bucket {
                        name: Some(bucket.to_string()),
                        ..Default::default()
                    },
                    object: S3Object {
                        key: Some(key.to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }],
        }
    }

    #[test]
    fn test_get_table_name_default() {
        // Clear env var if set
        std::env::remove_var("DYNAMODB_TABLE_NAME");
        assert_eq!(get_table_name(), "s3-dynamo-sync-table");
    }

    #[test]
    fn test_get_table_name_from_env() {
        std::env::set_var("DYNAMODB_TABLE_NAME", "custom-table");
        assert_eq!(get_table_name(), "custom-table");
        std::env::remove_var("DYNAMODB_TABLE_NAME");
    }

    #[test]
    fn test_create_event() {
        let event = create_test_event("test-bucket", "test-key.txt", "ObjectCreated:Put");
        assert_eq!(event.records.len(), 1);
        assert_eq!(
            event.records[0].s3.bucket.name,
            Some("test-bucket".to_string())
        );
    }
}