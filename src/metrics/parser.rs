//! Parser for promphp Redis format.
//!
//! promphp stores metrics in Redis with:
//! - `{prefix}:{TYPE}_METRIC_KEYS` - SET with metric key names
//! - `{prefix}:{type}:{name}` - HASH with:
//!   - `__meta` - JSON metadata
//!   - `base64(json(labelValues))` or `json(labelValues)` -> value

use super::types::{Metric, MetricType, Sample};
use crate::config::LabelFormat;
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
/// * `label_format` - Label encoding format (auto, json, or base64)
///
/// # Returns
/// Parsed Metric with all samples.
pub fn parse_promphp_metric(
    hash_data: HashMap<String, String>,
    label_format: LabelFormat,
) -> Result<Metric, ParseError> {
    // Extract and parse __meta
    let meta_json = hash_data
        .get("__meta")
        .ok_or_else(|| ParseError::InvalidMeta("missing __meta field".to_string()))?;

    let meta: MetricMeta =
        serde_json::from_str(meta_json).map_err(|e| ParseError::InvalidMeta(e.to_string()))?;

    let metric_type = parse_metric_type(&meta.metric_type)?;

    let mut metric = Metric::new(&meta.name, &meta.help, metric_type);

    if metric_type == MetricType::Histogram {
        // Histogram uses different field format in promphp:
        // each bucket is a separate hash field with JSON-object key
        let samples = parse_histogram_fields(&hash_data, &meta)?;
        for sample in samples {
            metric.add_sample(sample);
        }
    } else {
        // Counter/Gauge/Summary - each field has label values as key
        for (key, value) in hash_data.iter() {
            if key == "__meta" {
                continue;
            }

            let samples = parse_sample(key, value, &meta, label_format)?;
            for sample in samples {
                metric.add_sample(sample);
            }
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

fn parse_sample(
    key: &str,
    value: &str,
    meta: &MetricMeta,
    label_format: LabelFormat,
) -> Result<Vec<Sample>, ParseError> {
    // Decode label values based on format
    let label_values = decode_label_values(key, label_format)?;

    // Build labels map
    let labels: HashMap<String, String> = meta
        .label_names
        .iter()
        .zip(label_values.iter())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let val: f64 = value
        .parse()
        .map_err(|_| ParseError::InvalidValue(value.to_string()))?;
    Ok(vec![Sample::new(labels, val)])
}

fn decode_label_values(key: &str, format: LabelFormat) -> Result<Vec<String>, ParseError> {
    match format {
        LabelFormat::Json => decode_json_labels(key),
        LabelFormat::Base64 => decode_base64_labels(key),
        LabelFormat::Auto => {
            // Try JSON first, then base64
            if let Ok(values) = decode_json_labels(key) {
                return Ok(values);
            }
            decode_base64_labels(key)
        }
    }
}

fn decode_json_labels(key: &str) -> Result<Vec<String>, ParseError> {
    let values: Vec<serde_json::Value> =
        serde_json::from_str(key).map_err(|e| ParseError::InvalidLabelJson(e.to_string()))?;

    Ok(values
        .into_iter()
        .map(|v| match v {
            serde_json::Value::String(s) => s,
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "".to_string(),
            other => other.to_string(),
        })
        .collect())
}

fn decode_base64_labels(key: &str) -> Result<Vec<String>, ParseError> {
    let decoded = STANDARD
        .decode(key)
        .map_err(|e| ParseError::InvalidBase64(e.to_string()))?;

    let json_str =
        String::from_utf8(decoded).map_err(|e| ParseError::InvalidBase64(e.to_string()))?;

    let values: Vec<serde_json::Value> =
        serde_json::from_str(&json_str).map_err(|e| ParseError::InvalidLabelJson(e.to_string()))?;

    Ok(values
        .into_iter()
        .map(|v| match v {
            serde_json::Value::String(s) => s,
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "".to_string(),
            other => other.to_string(),
        })
        .collect())
}

/// Key format for histogram hash fields in promphp.
///
/// Each bucket is stored as a separate Redis hash field with a JSON-object key:
/// `{"b":2.5,"labelValues":["GET","route","/path",200]}`
///
/// `b` is a number for bucket bounds, or a string for "sum"/"+Inf".
#[derive(Debug, Deserialize)]
struct HistogramFieldKey {
    b: serde_json::Value,
    #[serde(rename = "labelValues")]
    label_values: Vec<serde_json::Value>,
}

/// Parse histogram fields from promphp Redis format.
///
/// Each hash field (except `__meta`) has a JSON-object key with bucket bound
/// and label values. Values are plain numbers (non-cumulative bucket counts).
fn parse_histogram_fields(
    hash_data: &HashMap<String, String>,
    meta: &MetricMeta,
) -> Result<Vec<Sample>, ParseError> {
    // Group entries by label_values
    // Key: serialized label_values, Value: (converted label strings, bucket data)
    let mut groups: HashMap<String, (Vec<String>, HashMap<String, f64>)> = HashMap::new();

    for (key, value) in hash_data.iter() {
        if key == "__meta" {
            continue;
        }

        let field_key: HistogramFieldKey =
            serde_json::from_str(key).map_err(|e| ParseError::InvalidLabelJson(e.to_string()))?;

        let val: f64 = value
            .parse()
            .map_err(|_| ParseError::InvalidValue(value.to_string()))?;

        let group_key = serde_json::to_string(&field_key.label_values)
            .map_err(|e| ParseError::InvalidLabelJson(e.to_string()))?;

        let bucket_key = match &field_key.b {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };

        let entry = groups.entry(group_key).or_insert_with(|| {
            let label_strs: Vec<String> = field_key
                .label_values
                .iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => "".to_string(),
                    other => other.to_string(),
                })
                .collect();
            (label_strs, HashMap::new())
        });

        entry.1.insert(bucket_key, val);
    }

    let mut samples = Vec::new();

    for (_group_key, (label_values, bucket_data)) in &groups {
        // Build base labels from meta.label_names + label_values
        let base_labels: HashMap<String, String> = meta
            .label_names
            .iter()
            .zip(label_values.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let sum_val = bucket_data.get("sum").copied().unwrap_or(0.0);
        let inf_count = bucket_data.get("+Inf").copied().unwrap_or(0.0);

        // Generate _bucket samples with cumulative counts
        let mut cumulative = 0.0;
        for bucket_bound in &meta.buckets {
            let bucket_key = format_bucket_key(*bucket_bound);
            let bucket_count = bucket_data.get(&bucket_key).copied().unwrap_or(0.0);
            cumulative += bucket_count;

            let mut bucket_labels = base_labels.clone();
            bucket_labels.insert("le".to_string(), format_le(*bucket_bound));
            samples.push(Sample::new(bucket_labels, cumulative).with_suffix("_bucket"));
        }

        // +Inf bucket (cumulative + observations beyond max bucket)
        let total_count = cumulative + inf_count;
        let mut inf_labels = base_labels.clone();
        inf_labels.insert("le".to_string(), "+Inf".to_string());
        samples.push(Sample::new(inf_labels, total_count).with_suffix("_bucket"));

        // _sum sample
        samples.push(Sample::new(base_labels.clone(), sum_val).with_suffix("_sum"));

        // _count sample (= total observations = +Inf bucket value)
        samples.push(Sample::new(base_labels.clone(), total_count).with_suffix("_count"));
    }

    Ok(samples)
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
