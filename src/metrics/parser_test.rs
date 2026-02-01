//! Tests for metrics parser.

use super::*;
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
    let values = decode_label_values(&encoded).unwrap();
    assert_eq!(values, vec!["value1", "value2"]);
}

#[test]
fn test_decode_empty_labels() {
    let encoded = STANDARD.encode("[]");
    let values = decode_label_values(&encoded).unwrap();
    assert!(values.is_empty());
}

#[test]
fn test_parse_counter() {
    let mut hash_data = HashMap::new();
    hash_data.insert(
        "__meta".to_string(),
        r#"{"name":"test_counter","help":"Test counter","type":"counter","labelNames":["method"]}"#.to_string(),
    );
    // base64 of ["GET"]
    let label_key = STANDARD.encode(r#"["GET"]"#);
    hash_data.insert(label_key, "42".to_string());

    let metric = parse_promphp_metric(hash_data).unwrap();
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
        r#"{"name":"memory_bytes","help":"Memory usage","type":"gauge","labelNames":["type"]}"#.to_string(),
    );
    let label_key = STANDARD.encode(r#"["heap"]"#);
    hash_data.insert(label_key, "1048576".to_string());

    let metric = parse_promphp_metric(hash_data).unwrap();
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

    let metric = parse_promphp_metric(hash_data).unwrap();
    assert_eq!(metric.name, "request_duration");
    assert_eq!(metric.metric_type, MetricType::Histogram);

    // Should have: 3 buckets + 1 +Inf + sum + count = 6 samples
    assert_eq!(metric.samples.len(), 6);

    // Check sum sample
    let sum_sample = metric.samples.iter().find(|s| s.suffix.as_deref() == Some("_sum")).unwrap();
    assert_eq!(sum_sample.value, 1.5);

    // Check count sample
    let count_sample = metric.samples.iter().find(|s| s.suffix.as_deref() == Some("_count")).unwrap();
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
    let result = parse_promphp_metric(hash_data);
    assert!(result.is_err());
}

#[test]
fn test_invalid_meta_json() {
    let mut hash_data = HashMap::new();
    hash_data.insert("__meta".to_string(), "not json".to_string());
    let result = parse_promphp_metric(hash_data);
    assert!(result.is_err());
}
