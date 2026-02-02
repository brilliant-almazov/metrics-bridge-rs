//! Tests for metrics parser.

use super::*;
use crate::config::LabelFormat;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::collections::HashMap;

#[test]
fn test_parse_metric_type() {
    assert_eq!(parse_metric_type("counter").unwrap(), MetricType::Counter);
    assert_eq!(parse_metric_type("GAUGE").unwrap(), MetricType::Gauge);
    assert_eq!(
        parse_metric_type("histogram").unwrap(),
        MetricType::Histogram
    );
    assert!(parse_metric_type("unknown").is_err());
}

#[test]
fn test_decode_label_values() {
    // base64 of ["value1", "value2"]
    let encoded = STANDARD.encode(r#"["value1","value2"]"#);
    let values = decode_label_values(&encoded, LabelFormat::Auto).unwrap();
    assert_eq!(values, vec!["value1", "value2"]);
}

#[test]
fn test_decode_empty_labels() {
    let encoded = STANDARD.encode("[]");
    let values = decode_label_values(&encoded, LabelFormat::Auto).unwrap();
    assert!(values.is_empty());
}

#[test]
fn test_parse_counter() {
    let mut hash_data = HashMap::new();
    hash_data.insert(
        "__meta".to_string(),
        r#"{"name":"test_counter","help":"Test counter","type":"counter","labelNames":["method"]}"#
            .to_string(),
    );
    // base64 of ["GET"]
    let label_key = STANDARD.encode(r#"["GET"]"#);
    hash_data.insert(label_key, "42".to_string());

    let metric = parse_promphp_metric(hash_data, LabelFormat::Auto).unwrap();
    assert_eq!(metric.name, "test_counter");
    assert_eq!(metric.metric_type, MetricType::Counter);
    assert_eq!(metric.samples.len(), 1);
    assert_eq!(metric.samples[0].value, 42.0);
    assert_eq!(metric.samples[0].labels.get("method").unwrap(), "GET");
}

#[test]
fn test_parse_gauge() {
    let mut hash_data = HashMap::new();
    hash_data.insert(
        "__meta".to_string(),
        r#"{"name":"memory_bytes","help":"Memory usage","type":"gauge","labelNames":["type"]}"#
            .to_string(),
    );
    let label_key = STANDARD.encode(r#"["heap"]"#);
    hash_data.insert(label_key, "1048576".to_string());

    let metric = parse_promphp_metric(hash_data, LabelFormat::Auto).unwrap();
    assert_eq!(metric.name, "memory_bytes");
    assert_eq!(metric.metric_type, MetricType::Gauge);
    assert_eq!(metric.samples[0].value, 1048576.0);
}

#[test]
fn test_parse_histogram() {
    let mut hash_data = HashMap::new();
    hash_data.insert(
        "__meta".to_string(),
        r#"{"name":"request_duration","help":"Duration","type":"histogram","labelNames":["endpoint"],"buckets":[0.01,0.05,0.1]}"#.to_string(),
    );
    let label_key = STANDARD.encode(r#"["/api"]"#);
    hash_data.insert(
        label_key,
        r#"{"sum":1.5,"count":100,"buckets":{"0.01":10,"0.05":50,"0.1":90}}"#.to_string(),
    );

    let metric = parse_promphp_metric(hash_data, LabelFormat::Auto).unwrap();
    assert_eq!(metric.name, "request_duration");
    assert_eq!(metric.metric_type, MetricType::Histogram);

    // Should have: 3 buckets + 1 +Inf + sum + count = 6 samples
    assert_eq!(metric.samples.len(), 6);

    // Check sum sample
    let sum_sample = metric
        .samples
        .iter()
        .find(|s| s.suffix.as_deref() == Some("_sum"))
        .unwrap();
    assert_eq!(sum_sample.value, 1.5);

    // Check count sample
    let count_sample = metric
        .samples
        .iter()
        .find(|s| s.suffix.as_deref() == Some("_count"))
        .unwrap();
    assert_eq!(count_sample.value, 100.0);
}

#[test]
fn test_format_bucket_key() {
    assert_eq!(format_bucket_key(0.01), "0.01");
    assert_eq!(format_bucket_key(1.0), "1");
    assert_eq!(format_bucket_key(10.0), "10");
}

#[test]
fn test_missing_meta() {
    let hash_data = HashMap::new();
    let result = parse_promphp_metric(hash_data, LabelFormat::Auto);
    assert!(result.is_err());
}

#[test]
fn test_invalid_meta_json() {
    let mut hash_data = HashMap::new();
    hash_data.insert("__meta".to_string(), "not json".to_string());
    let result = parse_promphp_metric(hash_data, LabelFormat::Auto);
    assert!(result.is_err());
}

#[test]
fn test_parse_summary() {
    let mut hash_data = HashMap::new();
    hash_data.insert(
        "__meta".to_string(),
        r#"{"name":"request_latency","help":"Request latency","type":"summary","labelNames":[]}"#
            .to_string(),
    );
    let label_key = STANDARD.encode("[]");
    hash_data.insert(label_key, "123.45".to_string());

    let metric = parse_promphp_metric(hash_data, LabelFormat::Auto).unwrap();
    assert_eq!(metric.name, "request_latency");
    assert_eq!(metric.metric_type, MetricType::Summary);
    assert_eq!(metric.samples[0].value, 123.45);
}

#[test]
fn test_parse_metric_type_unknown() {
    let result = parse_metric_type("unknown_type");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ParseError::UnknownType(_)));
    assert!(err.to_string().contains("unknown_type"));
}

#[test]
fn test_decode_label_values_raw_json() {
    // Test raw JSON format (not base64)
    let raw_json = r#"["value1","value2"]"#;
    let values = decode_label_values(raw_json, LabelFormat::Json).unwrap();
    assert_eq!(values, vec!["value1", "value2"]);
}

#[test]
fn test_decode_label_values_mixed_types() {
    // Test raw JSON with mixed types (strings + numbers) - common in promphp
    let raw_json = r#"["POST","api_endpoint","/v1/auth",401]"#;
    let values = decode_label_values(raw_json, LabelFormat::Json).unwrap();
    assert_eq!(values, vec!["POST", "api_endpoint", "/v1/auth", "401"]);
}

#[test]
fn test_parse_counter_with_mixed_label_types() {
    let mut hash_data = HashMap::new();
    hash_data.insert(
        "__meta".to_string(),
        r#"{"name":"http_requests","help":"HTTP requests","type":"counter","labelNames":["method","route","path","status"]}"#
            .to_string(),
    );
    // Raw JSON with mixed types (strings + number for status code)
    let label_key = r#"["GET","api_users","/v1/users",200]"#;
    hash_data.insert(label_key.to_string(), "42".to_string());

    let metric = parse_promphp_metric(hash_data, LabelFormat::Auto).unwrap();
    assert_eq!(metric.name, "http_requests");
    assert_eq!(metric.samples.len(), 1);
    assert_eq!(metric.samples[0].labels.get("method").unwrap(), "GET");
    assert_eq!(metric.samples[0].labels.get("status").unwrap(), "200");
}

#[test]
fn test_decode_label_values_invalid_base64() {
    let result = decode_label_values("not-valid-base64!!!", LabelFormat::Base64);
    assert!(result.is_err());
    assert!(matches!(result, Err(ParseError::InvalidBase64(_))));
}

#[test]
fn test_parse_invalid_value() {
    let mut hash_data = HashMap::new();
    hash_data.insert(
        "__meta".to_string(),
        r#"{"name":"test","help":"Test","type":"counter","labelNames":[]}"#.to_string(),
    );
    let label_key = STANDARD.encode("[]");
    hash_data.insert(label_key, "not-a-number".to_string());

    let result = parse_promphp_metric(hash_data, LabelFormat::Auto);
    assert!(result.is_err());
    assert!(matches!(result, Err(ParseError::InvalidValue(_))));
}

#[test]
fn test_format_le_large_integer() {
    // Large integer should format as integer
    assert_eq!(format_le(1000.0), "1000");
    assert_eq!(format_le(1000000.0), "1000000");
}

#[test]
fn test_format_le_decimal() {
    // Decimal should keep decimal format
    assert_eq!(format_le(0.5), "0.5");
    assert_eq!(format_le(1.5), "1.5");
}

#[test]
fn test_format_le_very_large() {
    // Very large numbers (>= 1e10) should use scientific notation
    let result = format_le(1e11);
    assert!(result.contains("e") || result.contains("E") || result == "100000000000");
}

#[test]
fn test_histogram_missing_bucket() {
    let mut hash_data = HashMap::new();
    hash_data.insert(
        "__meta".to_string(),
        r#"{"name":"hist","help":"Histogram","type":"histogram","labelNames":[],"buckets":[0.1,0.5,1.0]}"#.to_string(),
    );
    let label_key = STANDARD.encode("[]");
    // Only some buckets present - missing 0.5
    hash_data.insert(
        label_key,
        r#"{"sum":10.0,"count":50,"buckets":{"0.1":10,"1":40}}"#.to_string(),
    );

    let metric = parse_promphp_metric(hash_data, LabelFormat::Auto).unwrap();

    // Should still work, missing bucket defaults to 0
    assert_eq!(metric.samples.len(), 6); // 3 buckets + inf + sum + count
}

#[test]
fn test_parse_error_display() {
    let e = ParseError::InvalidMeta("test".to_string());
    assert!(e.to_string().contains("test"));

    let e = ParseError::InvalidBase64("base64".to_string());
    assert!(e.to_string().contains("base64"));

    let e = ParseError::InvalidLabelJson("json".to_string());
    assert!(e.to_string().contains("json"));

    let e = ParseError::InvalidValue("value".to_string());
    assert!(e.to_string().contains("value"));

    let e = ParseError::UnknownType("type".to_string());
    assert!(e.to_string().contains("type"));
}
