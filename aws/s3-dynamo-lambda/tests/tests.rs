use aws_sdk_dynamodb::types::AttributeValue;
use s3_dynamo_lambda::{attribute_to_string, Request};

#[test]
fn test_attribute_to_string_string() {
    let attr = AttributeValue::S("hello".to_string());
    assert_eq!(attribute_to_string(&attr), "hello");
}

#[test]
fn test_attribute_to_string_number() {
    let attr = AttributeValue::N("42".to_string());
    assert_eq!(attribute_to_string(&attr), "42");
}

#[test]
fn test_attribute_to_string_bool() {
    let attr = AttributeValue::Bool(true);
    assert_eq!(attribute_to_string(&attr), "true");
}

#[test]
fn test_attribute_to_string_null() {
    let attr = AttributeValue::Null(true);
    assert_eq!(attribute_to_string(&attr), "null");
}

#[test]
fn test_attribute_to_string_string_set() {
    let attr = AttributeValue::Ss(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(attribute_to_string(&attr), "[a, b]");
}

#[test]
fn test_attribute_to_string_number_set() {
    let attr = AttributeValue::Ns(vec!["1".to_string(), "2".to_string(), "3".to_string()]);
    assert_eq!(attribute_to_string(&attr), "[1, 2, 3]");
}

#[test]
fn test_attribute_to_string_list() {
    let attr = AttributeValue::L(vec![
        AttributeValue::S("hello".to_string()),
        AttributeValue::N("42".to_string()),
    ]);
    assert_eq!(attribute_to_string(&attr), "[hello, 42]");
}

#[test]
fn test_attribute_to_string_map() {
    let mut map = std::collections::HashMap::new();
    map.insert("name".to_string(), AttributeValue::S("test".to_string()));
    let attr = AttributeValue::M(map);
    assert_eq!(attribute_to_string(&attr), "{name: test}");
}

#[test]
fn test_request_deserialization() {
    let json = r#"{
        "csv_file_path": "/tmp/data.csv",
        "s3_bucket": "my-bucket",
        "s3_csv_key": "uploads/data.csv",
        "s3_results_key": "results/output.json",
        "dynamo_table": "my-table",
        "partition_key_name": "pk",
        "partition_key_value": "user123"
    }"#;

    let request: Request = serde_json::from_str(json).unwrap();
    assert_eq!(request.csv_file_path, "/tmp/data.csv");
    assert_eq!(request.s3_bucket, "my-bucket");
    assert_eq!(request.s3_csv_key, "uploads/data.csv");
    assert_eq!(request.s3_results_key, "results/output.json");
    assert_eq!(request.dynamo_table, "my-table");
    assert_eq!(request.partition_key_name, "pk");
    assert_eq!(request.partition_key_value, "user123");
    assert!(request.sort_key_name.is_none());
    assert!(request.sort_key_value.is_none());
    assert!(!request.create_test_file);
}

#[test]
fn test_request_with_sort_key() {
    let json = r#"{
        "csv_file_path": "/tmp/data.csv",
        "s3_bucket": "my-bucket",
        "s3_csv_key": "uploads/data.csv",
        "s3_results_key": "results/output.json",
        "dynamo_table": "my-table",
        "partition_key_name": "pk",
        "partition_key_value": "user123",
        "sort_key_name": "sk",
        "sort_key_value": "order456"
    }"#;

    let request: Request = serde_json::from_str(json).unwrap();
    assert_eq!(request.sort_key_name, Some("sk".to_string()));
    assert_eq!(request.sort_key_value, Some("order456".to_string()));
}

#[test]
fn test_request_with_create_test_file() {
    let json = r#"{
        "csv_file_path": "/tmp/data.csv",
        "s3_bucket": "my-bucket",
        "s3_csv_key": "uploads/data.csv",
        "s3_results_key": "results/output.json",
        "dynamo_table": "my-table",
        "partition_key_name": "pk",
        "partition_key_value": "user123",
        "create_test_file": true
    }"#;

    let request: Request = serde_json::from_str(json).unwrap();
    assert!(request.create_test_file);
}

#[test]
fn test_format_key_string_simple() {
    let request = Request {
        csv_file_path: String::new(),
        s3_bucket: String::new(),
        s3_csv_key: String::new(),
        s3_results_key: String::new(),
        dynamo_table: String::new(),
        partition_key_name: "pk".to_string(),
        partition_key_value: "user123".to_string(),
        sort_key_name: None,
        sort_key_value: None,
        create_test_file: false,
    };
    assert_eq!(request.format_key_string(), "pk=user123");
}

#[test]
fn test_format_key_string_composite() {
    let request = Request {
        csv_file_path: String::new(),
        s3_bucket: String::new(),
        s3_csv_key: String::new(),
        s3_results_key: String::new(),
        dynamo_table: String::new(),
        partition_key_name: "pk".to_string(),
        partition_key_value: "user123".to_string(),
        sort_key_name: Some("sk".to_string()),
        sort_key_value: Some("order456".to_string()),
        create_test_file: false,
    };
    assert_eq!(request.format_key_string(), "pk=user123, sk=order456");
}

#[test]
fn test_format_key_string_partial_sort_key() {
    // Only sort key name, no value - should fall back to simple format
    let request = Request {
        csv_file_path: String::new(),
        s3_bucket: String::new(),
        s3_csv_key: String::new(),
        s3_results_key: String::new(),
        dynamo_table: String::new(),
        partition_key_name: "pk".to_string(),
        partition_key_value: "user123".to_string(),
        sort_key_name: Some("sk".to_string()),
        sort_key_value: None,
        create_test_file: false,
    };
    assert_eq!(request.format_key_string(), "pk=user123");
}
