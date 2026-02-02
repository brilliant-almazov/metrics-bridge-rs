//! Tests for source registry.

use super::SourceRegistry;
use crate::metrics::{Metric, MetricFamily, MetricType, Sample};
use crate::source::{Source, SourceError, SourceResult};
use std::collections::HashMap;
use std::sync::Arc;

/// Mock source for testing.
#[derive(Debug)]
struct MockSource {
    name: String,
    should_fail: bool,
    healthy: bool,
}

impl MockSource {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            should_fail: false,
            healthy: true,
        }
    }

    fn failing(name: &str) -> Self {
        Self {
            name: name.to_string(),
            should_fail: true,
            healthy: false,
        }
    }
}

#[async_trait::async_trait]
impl Source for MockSource {
    fn name(&self) -> &str {
        &self.name
    }

    async fn collect(&self) -> SourceResult<MetricFamily> {
        if self.should_fail {
            return Err(SourceError::Connection("mock failure".to_string()));
        }

        let mut family = MetricFamily::new(&self.name);
        let mut metric = Metric::new(
            format!("{}_requests", self.name),
            "Mock metric",
            MetricType::Counter,
        );
        metric.add_sample(Sample::new(HashMap::new(), 42.0));
        family.add_metric(metric);

        Ok(family)
    }

    async fn health_check(&self) -> bool {
        self.healthy
    }
}

#[test]
fn test_registry_new() {
    let registry = SourceRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_registry_default() {
    let registry = SourceRegistry::default();
    assert!(registry.is_empty());
}

#[test]
fn test_registry_empty() {
    let registry = SourceRegistry::new();

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_registry_with_sources() {
    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(MockSource::new("source1")),
        Arc::new(MockSource::new("source2")),
    ];

    let registry = SourceRegistry::with_sources(sources);

    assert!(!registry.is_empty());
    assert_eq!(registry.len(), 2);
}

#[tokio::test]
async fn test_registry_collect_all_success() {
    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(MockSource::new("source1")),
        Arc::new(MockSource::new("source2")),
    ];

    let registry = SourceRegistry::with_sources(sources);

    let (families, errors) = registry.collect_all().await;

    assert_eq!(families.len(), 2);
    assert!(errors.is_empty());
}

#[tokio::test]
async fn test_registry_collect_all_with_failures() {
    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(MockSource::new("healthy")),
        Arc::new(MockSource::failing("failing")),
    ];

    let registry = SourceRegistry::with_sources(sources);

    let (families, errors) = registry.collect_all().await;

    assert_eq!(families.len(), 1);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, "failing");
}

#[tokio::test]
async fn test_registry_collect_all_empty() {
    let registry = SourceRegistry::new();

    let (families, errors) = registry.collect_all().await;

    assert!(families.is_empty());
    assert!(errors.is_empty());
}

#[tokio::test]
async fn test_registry_health_check_all_healthy() {
    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(MockSource::new("source1")),
        Arc::new(MockSource::new("source2")),
    ];

    let registry = SourceRegistry::with_sources(sources);

    let results = registry.health_check_all().await;

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|(_, healthy)| *healthy));
}

#[tokio::test]
async fn test_registry_health_check_mixed() {
    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(MockSource::new("healthy")),
        Arc::new(MockSource::failing("unhealthy")),
    ];

    let registry = SourceRegistry::with_sources(sources);

    let results = registry.health_check_all().await;

    assert_eq!(results.len(), 2);

    let healthy_count = results.iter().filter(|(_, h)| *h).count();
    let unhealthy_count = results.iter().filter(|(_, h)| !*h).count();

    assert_eq!(healthy_count, 1);
    assert_eq!(unhealthy_count, 1);
}

#[tokio::test]
async fn test_registry_health_check_empty() {
    let registry = SourceRegistry::new();

    let results = registry.health_check_all().await;

    assert!(results.is_empty());
}

#[tokio::test]
async fn test_registry_all_healthy_true() {
    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(MockSource::new("source1")),
        Arc::new(MockSource::new("source2")),
    ];

    let registry = SourceRegistry::with_sources(sources);

    assert!(registry.all_healthy().await);
}

#[tokio::test]
async fn test_registry_all_healthy_false() {
    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(MockSource::new("healthy")),
        Arc::new(MockSource::failing("unhealthy")),
    ];

    let registry = SourceRegistry::with_sources(sources);

    assert!(!registry.all_healthy().await);
}

#[tokio::test]
async fn test_registry_all_healthy_empty() {
    let registry = SourceRegistry::new();

    // Empty registry is considered healthy
    assert!(registry.all_healthy().await);
}

#[tokio::test]
async fn test_registry_collect_returns_metric_families() {
    let sources: Vec<Arc<dyn Source>> = vec![Arc::new(MockSource::new("test"))];

    let registry = SourceRegistry::with_sources(sources);

    let (families, _) = registry.collect_all().await;

    assert_eq!(families.len(), 1);
    assert_eq!(families[0].source, "test");
    assert_eq!(families[0].metrics.len(), 1);
    assert_eq!(families[0].metrics[0].name, "test_requests");
}

#[test]
fn test_registry_debug() {
    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(MockSource::new("source1")),
        Arc::new(MockSource::new("source2")),
    ];

    let registry = SourceRegistry::with_sources(sources);

    let debug_str = format!("{:?}", registry);
    assert!(debug_str.contains("SourceRegistry"));
    assert!(debug_str.contains("sources_count: 2"));
}

#[test]
fn test_source_error_display() {
    let e = SourceError::Connection("failed".to_string());
    assert!(e.to_string().contains("failed"));

    let e = SourceError::Parse("parse error".to_string());
    assert!(e.to_string().contains("parse error"));

    let e = SourceError::Timeout;
    assert!(e.to_string().contains("Timeout"));

    let e = SourceError::Redis("redis error".to_string());
    assert!(e.to_string().contains("redis error"));
}

#[test]
fn test_source_error_debug() {
    let e = SourceError::Connection("test".to_string());
    let debug = format!("{:?}", e);
    assert!(debug.contains("Connection"));
}

#[tokio::test]
async fn test_registry_collect_all_all_fail() {
    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(MockSource::failing("fail1")),
        Arc::new(MockSource::failing("fail2")),
    ];

    let registry = SourceRegistry::with_sources(sources);

    let (families, errors) = registry.collect_all().await;

    assert!(families.is_empty());
    assert_eq!(errors.len(), 2);
}

#[tokio::test]
async fn test_registry_all_healthy_single_unhealthy() {
    let sources: Vec<Arc<dyn Source>> = vec![Arc::new(MockSource::failing("unhealthy"))];

    let registry = SourceRegistry::with_sources(sources);

    assert!(!registry.all_healthy().await);
}

#[tokio::test]
async fn test_registry_metrics_content() {
    let sources: Vec<Arc<dyn Source>> = vec![Arc::new(MockSource::new("myapp"))];

    let registry = SourceRegistry::with_sources(sources);

    let (families, errors) = registry.collect_all().await;

    assert!(errors.is_empty());
    assert_eq!(families.len(), 1);

    let family = &families[0];
    assert_eq!(family.source, "myapp");
    assert_eq!(family.metrics.len(), 1);

    let metric = &family.metrics[0];
    assert_eq!(metric.name, "myapp_requests");
    assert_eq!(metric.samples.len(), 1);
    assert_eq!(metric.samples[0].value, 42.0);
}
