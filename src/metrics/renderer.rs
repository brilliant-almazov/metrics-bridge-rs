//! Prometheus text exposition format renderer.
//!
//! Renders MetricFamily collections to Prometheus text format.
//! See: https://prometheus.io/docs/instrumenting/exposition_formats/

use super::types::MetricFamily;
use std::collections::HashMap;
use std::fmt::Write;

/// Render metrics to Prometheus text exposition format.
pub fn render_metrics(families: &[MetricFamily]) -> String {
    let mut output = String::new();
    let mut seen_metrics: std::collections::HashSet<String> = std::collections::HashSet::new();

    for family in families {
        for metric in &family.metrics {
            let full_name = &metric.name;

            // Only output HELP and TYPE once per metric name
            if !seen_metrics.contains(full_name) {
                // HELP line
                writeln!(
                    output,
                    "# HELP {} {}",
                    full_name,
                    escape_help(&metric.help)
                )
                .unwrap();

                // TYPE line
                writeln!(output, "# TYPE {} {}", full_name, metric.metric_type).unwrap();

                seen_metrics.insert(full_name.clone());
            }

            // Sample lines
            for sample in &metric.samples {
                let metric_name = match &sample.suffix {
                    Some(suffix) => format!("{}{}", full_name, suffix),
                    None => full_name.clone(),
                };

                // Merge extra labels from family
                let mut all_labels = family.extra_labels.clone();
                all_labels.extend(sample.labels.clone());

                let labels_str = format_labels(&all_labels);

                if labels_str.is_empty() {
                    writeln!(output, "{} {}", metric_name, format_value(sample.value)).unwrap();
                } else {
                    writeln!(
                        output,
                        "{}{{{}}} {}",
                        metric_name,
                        labels_str,
                        format_value(sample.value)
                    )
                    .unwrap();
                }
            }
        }
    }

    output
}

/// Escape help text for Prometheus format.
fn escape_help(help: &str) -> String {
    help.replace('\\', "\\\\").replace('\n', "\\n")
}

/// Format labels as key="value",key="value".
fn format_labels(labels: &HashMap<String, String>) -> String {
    if labels.is_empty() {
        return String::new();
    }

    let mut pairs: Vec<_> = labels
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", k, escape_label_value(v)))
        .collect();

    // Sort for deterministic output
    pairs.sort();
    pairs.join(",")
}

/// Escape label value for Prometheus format.
fn escape_label_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Format metric value.
fn format_value(value: f64) -> String {
    if value.is_nan() {
        "NaN".to_string()
    } else if value.is_infinite() {
        if value.is_sign_positive() {
            "+Inf".to_string()
        } else {
            "-Inf".to_string()
        }
    } else if value == value.floor() && value.abs() < 1e15 {
        // Integer value
        format!("{:.0}", value)
    } else {
        format!("{}", value)
    }
}

#[cfg(test)]
#[path = "renderer_test.rs"]
mod tests;
