//! Tests for metric types.

use super::*;
use std::collections::HashMap;

#[test]
fn test_metric_type_as_str() {
    assert_eq!(MetricType::Counter.as_str(), "counter");
    assert_eq!(MetricType::Gauge.as_str(), "gauge");
    assert_eq!(MetricType::Histogram.as_str(), "histogram");
    assert_eq!(MetricType::Summary.as_str(), "summary");
}

#[test]
fn test_metric_type_display() {
    assert_eq!(format!("{}", MetricType::Counter), "counter");
    assert_eq!(format!("{}", MetricType::Gauge), "gauge");
    assert_eq!(format!("{}", MetricType::Histogram), "histogram");
    assert_eq!(format!("{}", MetricType::Summary), "summary");
}

#[test]
fn test_metric_type_equality() {
    assert_eq!(MetricType::Counter, MetricType::Counter);
    assert_ne!(MetricType::Counter, MetricType::Gauge);
}

#[test]
fn test_sample_new() {
    let mut labels = HashMap::new();
    labels.insert("method".to_string(), "GET".to_string());

    let sample = Sample::new(labels.clone(), 42.0);

    assert_eq!(sample.labels.get("method"), Some(&"GET".to_string()));
    assert_eq!(sample.value, 42.0);
    assert!(sample.suffix.is_none());
}

#[test]
fn test_sample_with_suffix() {
    let sample = Sample::new(HashMap::new(), 1.0).with_suffix("_bucket");

    assert_eq!(sample.suffix, Some("_bucket".to_string()));
}

#[test]
fn test_sample_with_suffix_chained() {
    let sample = Sample::new(HashMap::new(), 5.0).with_suffix("_count");

    assert_eq!(sample.value, 5.0);
    assert_eq!(sample.suffix, Some("_count".to_string()));
}

#[test]
fn test_sample_clone() {
    let mut labels = HashMap::new();
    labels.insert("key".to_string(), "value".to_string());

    let sample = Sample::new(labels, 99.9).with_suffix("_sum");
    let cloned = sample.clone();

    assert_eq!(cloned.labels.get("key"), Some(&"value".to_string()));
    assert_eq!(cloned.value, 99.9);
    assert_eq!(cloned.suffix, Some("_sum".to_string()));
}

#[test]
fn test_metric_new() {
    let metric = Metric::new("http_requests", "Total HTTP requests", MetricType::Counter);

    assert_eq!(metric.name, "http_requests");
    assert_eq!(metric.help, "Total HTTP requests");
    assert_eq!(metric.metric_type, MetricType::Counter);
    assert!(metric.samples.is_empty());
}

#[test]
fn test_metric_add_sample() {
    let mut metric = Metric::new("gauge_metric", "A gauge", MetricType::Gauge);

    metric.add_sample(Sample::new(HashMap::new(), 1.0));
    metric.add_sample(Sample::new(HashMap::new(), 2.0));

    assert_eq!(metric.samples.len(), 2);
    assert_eq!(metric.samples[0].value, 1.0);
    assert_eq!(metric.samples[1].value, 2.0);
}

#[test]
fn test_metric_clone() {
    let mut metric = Metric::new("test", "test metric", MetricType::Histogram);
    metric.add_sample(Sample::new(HashMap::new(), 5.0));

    let cloned = metric.clone();

    assert_eq!(cloned.name, "test");
    assert_eq!(cloned.samples.len(), 1);
}

#[test]
fn test_metric_family_new() {
    let family = MetricFamily::new("redis-source");

    assert_eq!(family.source, "redis-source");
    assert!(family.extra_labels.is_empty());
    assert!(family.metrics.is_empty());
}

#[test]
fn test_metric_family_default() {
    let family = MetricFamily::default();

    assert_eq!(family.source, "");
    assert!(family.extra_labels.is_empty());
    assert!(family.metrics.is_empty());
}

#[test]
fn test_metric_family_with_labels() {
    let mut labels = HashMap::new();
    labels.insert("env".to_string(), "prod".to_string());
    labels.insert("region".to_string(), "us-east".to_string());

    let family = MetricFamily::new("source").with_labels(labels);

    assert_eq!(family.extra_labels.len(), 2);
    assert_eq!(family.extra_labels.get("env"), Some(&"prod".to_string()));
    assert_eq!(
        family.extra_labels.get("region"),
        Some(&"us-east".to_string())
    );
}

#[test]
fn test_metric_family_add_metric() {
    let mut family = MetricFamily::new("test-source");

    family.add_metric(Metric::new("metric1", "first", MetricType::Counter));
    family.add_metric(Metric::new("metric2", "second", MetricType::Gauge));

    assert_eq!(family.metrics.len(), 2);
    assert_eq!(family.metrics[0].name, "metric1");
    assert_eq!(family.metrics[1].name, "metric2");
}

#[test]
fn test_metric_family_clone() {
    let mut labels = HashMap::new();
    labels.insert("key".to_string(), "value".to_string());

    let mut family = MetricFamily::new("src").with_labels(labels);
    family.add_metric(Metric::new("m", "h", MetricType::Gauge));

    let cloned = family.clone();

    assert_eq!(cloned.source, "src");
    assert_eq!(cloned.extra_labels.len(), 1);
    assert_eq!(cloned.metrics.len(), 1);
}

#[test]
fn test_sample_empty_labels() {
    let sample = Sample::new(HashMap::new(), 0.0);

    assert!(sample.labels.is_empty());
    assert_eq!(sample.value, 0.0);
}

#[test]
fn test_metric_with_special_values() {
    let mut metric = Metric::new("special", "special values", MetricType::Gauge);

    metric.add_sample(Sample::new(HashMap::new(), f64::INFINITY));
    metric.add_sample(Sample::new(HashMap::new(), f64::NEG_INFINITY));
    metric.add_sample(Sample::new(HashMap::new(), f64::NAN));

    assert_eq!(metric.samples.len(), 3);
    assert!(metric.samples[0].value.is_infinite());
    assert!(metric.samples[1].value.is_infinite());
    assert!(metric.samples[2].value.is_nan());
}
