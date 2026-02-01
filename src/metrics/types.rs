//! Metric type definitions.

use std::collections::HashMap;

/// Metric type as defined by Prometheus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

impl MetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
            MetricType::Summary => "summary",
        }
    }
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single metric sample with labels and value.
#[derive(Debug, Clone)]
pub struct Sample {
    /// Label key-value pairs.
    pub labels: HashMap<String, String>,
    /// Metric value.
    pub value: f64,
    /// Optional suffix for histogram buckets (_bucket, _count, _sum).
    pub suffix: Option<String>,
}

impl Sample {
    pub fn new(labels: HashMap<String, String>, value: f64) -> Self {
        Self {
            labels,
            value,
            suffix: None,
        }
    }

    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }
}

/// A single metric with metadata.
#[derive(Debug, Clone)]
pub struct Metric {
    /// Metric name.
    pub name: String,
    /// Help text.
    pub help: String,
    /// Metric type.
    pub metric_type: MetricType,
    /// All samples for this metric.
    pub samples: Vec<Sample>,
}

impl Metric {
    pub fn new(name: impl Into<String>, help: impl Into<String>, metric_type: MetricType) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            metric_type,
            samples: Vec::new(),
        }
    }

    pub fn add_sample(&mut self, sample: Sample) {
        self.samples.push(sample);
    }
}

/// A collection of metrics from a single source.
#[derive(Debug, Clone, Default)]
pub struct MetricFamily {
    /// Source name.
    pub source: String,
    /// Additional labels to add to all metrics.
    pub extra_labels: HashMap<String, String>,
    /// Metrics in this family.
    pub metrics: Vec<Metric>,
}

impl MetricFamily {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            extra_labels: HashMap::new(),
            metrics: Vec::new(),
        }
    }

    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.extra_labels = labels;
        self
    }

    pub fn add_metric(&mut self, metric: Metric) {
        self.metrics.push(metric);
    }
}
