//! Tests for metrics renderer.

use super::*;
use crate::metrics::types::{Metric, MetricType, Sample};
use std::collections::HashMap;

#[test]
fn test_escape_help() {
    assert_eq!(escape_help("simple help"), "simple help");
    assert_eq!(escape_help("line1\nline2"), "line1\\nline2");
    assert_eq!(escape_help("back\\slash"), "back\\\\slash");
}

#[test]
fn test_escape_label_value() {
    assert_eq!(escape_label_value("simple"), "simple");
    assert_eq!(escape_label_value("with\"quote"), "with\\\"quote");
    assert_eq!(escape_label_value("with\nnewline"), "with\\nnewline");
}

#[test]
fn test_format_value() {
    assert_eq!(format_value(42.0), "42");
    assert_eq!(format_value(3.15), "3.15");
    assert_eq!(format_value(f64::NAN), "NaN");
    assert_eq!(format_value(f64::INFINITY), "+Inf");
    assert_eq!(format_value(f64::NEG_INFINITY), "-Inf");
}

#[test]
fn test_format_value_large_integer() {
    assert_eq!(format_value(1000000.0), "1000000");
}

#[test]
fn test_render_simple_counter() {
    let mut metric = Metric::new("http_requests_total", "Total HTTP requests", MetricType::Counter);
    let mut labels = HashMap::new();
    labels.insert("method".to_string(), "GET".to_string());
    metric.add_sample(Sample::new(labels, 100.0));

    let family = MetricFamily {
        source: "test".to_string(),
        extra_labels: HashMap::new(),
        metrics: vec![metric],
    };

    let output = render_metrics(&[family]);
    assert!(output.contains("# HELP http_requests_total Total HTTP requests"));
    assert!(output.contains("# TYPE http_requests_total counter"));
    assert!(output.contains("http_requests_total{method=\"GET\"} 100"));
}

#[test]
fn test_render_with_extra_labels() {
    let mut metric = Metric::new("test_metric", "Test", MetricType::Gauge);
    metric.add_sample(Sample::new(HashMap::new(), 42.0));

    let mut extra_labels = HashMap::new();
    extra_labels.insert("app".to_string(), "web".to_string());

    let family = MetricFamily {
        source: "test".to_string(),
        extra_labels,
        metrics: vec![metric],
    };

    let output = render_metrics(&[family]);
    assert!(output.contains("test_metric{app=\"web\"} 42"));
}

#[test]
fn test_render_no_labels() {
    let mut metric = Metric::new("simple_gauge", "Simple", MetricType::Gauge);
    metric.add_sample(Sample::new(HashMap::new(), 1.0));

    let family = MetricFamily {
        source: "test".to_string(),
        extra_labels: HashMap::new(),
        metrics: vec![metric],
    };

    let output = render_metrics(&[family]);
    assert!(output.contains("simple_gauge 1"));
}

#[test]
fn test_render_multiple_families() {
    let mut metric1 = Metric::new("metric_a", "A", MetricType::Counter);
    metric1.add_sample(Sample::new(HashMap::new(), 1.0));

    let mut metric2 = Metric::new("metric_b", "B", MetricType::Gauge);
    metric2.add_sample(Sample::new(HashMap::new(), 2.0));

    let family1 = MetricFamily {
        source: "source1".to_string(),
        extra_labels: HashMap::new(),
        metrics: vec![metric1],
    };

    let family2 = MetricFamily {
        source: "source2".to_string(),
        extra_labels: HashMap::new(),
        metrics: vec![metric2],
    };

    let output = render_metrics(&[family1, family2]);
    assert!(output.contains("metric_a 1"));
    assert!(output.contains("metric_b 2"));
}

#[test]
fn test_render_histogram_with_suffix() {
    let mut metric = Metric::new("request_duration", "Duration", MetricType::Histogram);

    let mut labels = HashMap::new();
    labels.insert("endpoint".to_string(), "/api".to_string());

    metric.add_sample(Sample::new(labels.clone(), 1.5).with_suffix("_sum"));
    metric.add_sample(Sample::new(labels.clone(), 100.0).with_suffix("_count"));

    let mut bucket_labels = labels.clone();
    bucket_labels.insert("le".to_string(), "0.1".to_string());
    metric.add_sample(Sample::new(bucket_labels, 50.0).with_suffix("_bucket"));

    let family = MetricFamily {
        source: "test".to_string(),
        extra_labels: HashMap::new(),
        metrics: vec![metric],
    };

    let output = render_metrics(&[family]);
    assert!(output.contains("request_duration_sum{endpoint=\"/api\"} 1.5"));
    assert!(output.contains("request_duration_count{endpoint=\"/api\"} 100"));
    assert!(output.contains("request_duration_bucket{endpoint=\"/api\",le=\"0.1\"} 50"));
}

#[test]
fn test_labels_sorted_deterministically() {
    let mut labels = HashMap::new();
    labels.insert("z".to_string(), "1".to_string());
    labels.insert("a".to_string(), "2".to_string());
    labels.insert("m".to_string(), "3".to_string());

    let formatted = format_labels(&labels);
    assert_eq!(formatted, "a=\"2\",m=\"3\",z=\"1\"");
}
