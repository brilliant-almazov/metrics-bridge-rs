//! Tests for source registry.

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

/// Test registry that accepts mock sources.
struct TestRegistry {
    sources: Vec<Arc<dyn Source>>,
}

impl TestRegistry {
    fn new() -> Self {
        Self { sources: vec![] }
    }

    fn add(&mut self, source: impl Source + 'static) {
        self.sources.push(Arc::new(source));
    }

    fn len(&self) -> usize {
        self.sources.len()
    }

    fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    async fn collect_all(&self) -> (Vec<MetricFamily>, Vec<(String, SourceError)>) {
        let futures: Vec<_> = self
            .sources
            .iter()
            .map(|source| {
                let source = Arc::clone(source);
                async move {
                    let name = source.name().to_string();
                    let result = source.collect().await;
                    (name, result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut families = Vec::new();
        let mut errors = Vec::new();

        for (name, result) in results {
            match result {
                Ok(family) => families.push(family),
                Err(e) => errors.push((name, e)),
            }
        }

        (families, errors)
    }

    async fn health_check_all(&self) -> Vec<(String, bool)> {
        let futures: Vec<_> = self
            .sources
            .iter()
            .map(|source| {
                let source = Arc::clone(source);
                async move {
                    let name = source.name().to_string();
                    let healthy = source.health_check().await;
                    (name, healthy)
                }
            })
            .collect();

        futures::future::join_all(futures).await
    }
}

#[tokio::test]
async fn test_registry_empty() {
    let registry = TestRegistry::new();

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[tokio::test]
async fn test_registry_add_sources() {
    let mut registry = TestRegistry::new();

    registry.add(MockSource::new("source1"));
    registry.add(MockSource::new("source2"));

    assert!(!registry.is_empty());
    assert_eq!(registry.len(), 2);
}

#[tokio::test]
async fn test_registry_collect_all_success() {
    let mut registry = TestRegistry::new();
    registry.add(MockSource::new("source1"));
    registry.add(MockSource::new("source2"));

    let (families, errors) = registry.collect_all().await;

    assert_eq!(families.len(), 2);
    assert!(errors.is_empty());
}

#[tokio::test]
async fn test_registry_collect_all_with_failures() {
    let mut registry = TestRegistry::new();
    registry.add(MockSource::new("healthy"));
    registry.add(MockSource::failing("failing"));

    let (families, errors) = registry.collect_all().await;

    assert_eq!(families.len(), 1);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, "failing");
}

#[tokio::test]
async fn test_registry_health_check_all_healthy() {
    let mut registry = TestRegistry::new();
    registry.add(MockSource::new("source1"));
    registry.add(MockSource::new("source2"));

    let results = registry.health_check_all().await;

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|(_, healthy)| *healthy));
}

#[tokio::test]
async fn test_registry_health_check_mixed() {
    let mut registry = TestRegistry::new();
    registry.add(MockSource::new("healthy"));
    registry.add(MockSource::failing("unhealthy"));

    let results = registry.health_check_all().await;

    assert_eq!(results.len(), 2);

    let healthy_count = results.iter().filter(|(_, h)| *h).count();
    let unhealthy_count = results.iter().filter(|(_, h)| !*h).count();

    assert_eq!(healthy_count, 1);
    assert_eq!(unhealthy_count, 1);
}

#[tokio::test]
async fn test_registry_collect_returns_metric_families() {
    let mut registry = TestRegistry::new();
    registry.add(MockSource::new("test"));

    let (families, _) = registry.collect_all().await;

    assert_eq!(families.len(), 1);
    assert_eq!(families[0].source, "test");
    assert_eq!(families[0].metrics.len(), 1);
    assert_eq!(families[0].metrics[0].name, "test_requests");
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
