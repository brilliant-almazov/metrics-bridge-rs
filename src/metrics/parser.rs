//! Parser for promphp Redis format.
//!
//! promphp stores metrics in Redis with:
//! - `{prefix}:{TYPE}_METRIC_KEYS` - SET with metric key names
//! - `{prefix}:{type}:{name}` - HASH with:
//!   - `__meta` - JSON metadata
//!   - `base64(json(labelValues))` -> value

use super::types::{Metric, MetricType, Sample};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid JSON in __meta: {0}")]
    InvalidMeta(String),

    #[error("Invalid base64 in label key: {0}")]
    InvalidBase64(String),

    #[error("Invalid JSON in label values: {0}")]
    InvalidLabelJson(String),

    #[error("Invalid metric value: {0}")]
    InvalidValue(String),

    #[error("Unknown metric type: {0}")]
    UnknownType(String),
}

/// Metadata stored in __meta field.
#[derive(Debug, Deserialize)]
pub struct MetricMeta {
    pub name: String,
    pub help: String,
    #[serde(rename = "type")]
    pub metric_type: String,
    #[serde(rename = "labelNames")]
    pub label_names: Vec<String>,
    #[serde(default)]
    pub buckets: Vec<f64>,
}

/// Parse a promphp metric from Redis hash data.
///
/// # Arguments
/// * `hash_data` - HashMap from Redis HGETALL (field -> value)
///
/// # Returns
/// Parsed Metric with all samples.
pub fn parse_promphp_metric(hash_data: HashMap<String, String>) -> Result<Metric, ParseError> {
    // Extract and parse __meta
    let meta_json = hash_data
        .get("__meta")
        .ok_or_else(|| ParseError::InvalidMeta("missing __meta field".to_string()))?;

    let meta: MetricMeta =
        serde_json::from_str(meta_json).map_err(|e| ParseError::InvalidMeta(e.to_string()))?;

    let metric_type = parse_metric_type(&meta.metric_type)?;

    let mut metric = Metric::new(&meta.name, &meta.help, metric_type);

    // Parse each sample (skip __meta)
    for (key, value) in hash_data.iter() {
        if key == "__meta" {
            continue;
        }

        let samples = parse_sample(key, value, &meta)?;
        for sample in samples {
            metric.add_sample(sample);
        }
    }

    Ok(metric)
}

fn parse_metric_type(type_str: &str) -> Result<MetricType, ParseError> {
    match type_str.to_lowercase().as_str() {
        "counter" => Ok(MetricType::Counter),
        "gauge" => Ok(MetricType::Gauge),
        "histogram" => Ok(MetricType::Histogram),
        "summary" => Ok(MetricType::Summary),
        _ => Err(ParseError::UnknownType(type_str.to_string())),
    }
}

fn parse_sample(key: &str, value: &str, meta: &MetricMeta) -> Result<Vec<Sample>, ParseError> {
    // Decode base64 key to get label values JSON
    let label_values = decode_label_values(key)?;

    // Build labels map
    let labels: HashMap<String, String> = meta
        .label_names
        .iter()
        .zip(label_values.iter())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    match meta.metric_type.to_lowercase().as_str() {
        "histogram" => parse_histogram_sample(value, labels, &meta.buckets),
        _ => {
            // Counter/Gauge - simple value
            let val: f64 = value
                .parse()
                .map_err(|_| ParseError::InvalidValue(value.to_string()))?;
            Ok(vec![Sample::new(labels, val)])
        }
    }
}

fn decode_label_values(key: &str) -> Result<Vec<String>, ParseError> {
    // First try parsing as raw JSON (promphp can store labels as raw JSON arrays)
    // Labels may contain mixed types (strings, numbers), so parse as Value first
    if let Ok(values) = serde_json::from_str::<Vec<serde_json::Value>>(key) {
        let string_values: Vec<String> = values
            .into_iter()
            .map(|v| match v {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "".to_string(),
                other => other.to_string(),
            })
            .collect();
        return Ok(string_values);
    }

    // Fall back to base64 decoding (older promphp format)
    let decoded = STANDARD
        .decode(key)
        .map_err(|e| ParseError::InvalidBase64(e.to_string()))?;

    let json_str =
        String::from_utf8(decoded).map_err(|e| ParseError::InvalidBase64(e.to_string()))?;

    // Parse JSON array - also handle mixed types
    if let Ok(values) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
        let string_values: Vec<String> = values
            .into_iter()
            .map(|v| match v {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "".to_string(),
                other => other.to_string(),
            })
            .collect();
        return Ok(string_values);
    }

    // Try parsing as string array for backwards compatibility
    let values: Vec<String> =
        serde_json::from_str(&json_str).map_err(|e| ParseError::InvalidLabelJson(e.to_string()))?;

    Ok(values)
}

/// Parse histogram sample value (JSON object with bucket counts, sum, count).
fn parse_histogram_sample(
    value: &str,
    base_labels: HashMap<String, String>,
    buckets: &[f64],
) -> Result<Vec<Sample>, ParseError> {
    // promphp histogram value format:
    // {"sum": 123.45, "count": 100, "buckets": {"0.005": 10, "0.01": 20, ...}}
    let hist_data: HistogramValue =
        serde_json::from_str(value).map_err(|e| ParseError::InvalidValue(e.to_string()))?;

    let mut samples = Vec::new();

    // _sum sample
    samples.push(Sample::new(base_labels.clone(), hist_data.sum).with_suffix("_sum"));

    // _count sample
    samples.push(Sample::new(base_labels.clone(), hist_data.count as f64).with_suffix("_count"));

    // _bucket samples
    let mut cumulative = 0u64;
    for bucket_bound in buckets {
        let bucket_key = format_bucket_key(*bucket_bound);
        let bucket_count = hist_data.buckets.get(&bucket_key).copied().unwrap_or(0);
        cumulative += bucket_count;

        let mut bucket_labels = base_labels.clone();
        bucket_labels.insert("le".to_string(), format_le(*bucket_bound));
        samples.push(Sample::new(bucket_labels, cumulative as f64).with_suffix("_bucket"));
    }

    // +Inf bucket
    let mut inf_labels = base_labels.clone();
    inf_labels.insert("le".to_string(), "+Inf".to_string());
    samples.push(Sample::new(inf_labels, hist_data.count as f64).with_suffix("_bucket"));

    Ok(samples)
}

#[derive(Debug, Deserialize)]
struct HistogramValue {
    sum: f64,
    count: u64,
    #[serde(default)]
    buckets: HashMap<String, u64>,
}

fn format_bucket_key(bound: f64) -> String {
    if bound == bound.floor() {
        format!("{:.0}", bound)
    } else {
        format!("{}", bound)
    }
}

fn format_le(bound: f64) -> String {
    if bound == bound.floor() && bound.abs() < 1e10 {
        format!("{:.0}", bound)
    } else {
        format!("{}", bound)
    }
}

#[cfg(test)]
#[path = "parser_test.rs"]
mod tests;
