//! Source registry for managing multiple metric sources.

use super::{PromphpRedisSource, Source, SourceError};
use crate::config::{Config, SourceType};
use crate::metrics::MetricFamily;
use futures::future::join_all;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Registry holding all configured metric sources.
pub struct SourceRegistry {
    sources: Vec<Arc<dyn Source>>,
}

impl SourceRegistry {
    /// Creates a new registry from configuration.
    pub fn from_config(config: &Config) -> Result<Self, SourceError> {
        let mut sources: Vec<Arc<dyn Source>> = Vec::new();

        for source_config in &config.sources {
            info!(
                name = %source_config.name,
                source_type = %source_config.source_type,
                "Initializing source"
            );

            let source: Arc<dyn Source> = match source_config.source_type {
                SourceType::PromphpRedis => {
                    Arc::new(PromphpRedisSource::new(source_config)?)
                }
            };

            sources.push(source);
        }

        Ok(Self { sources })
    }

    /// Returns the number of registered sources.
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// Returns true if no sources are registered.
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// Collects metrics from all sources in parallel.
    ///
    /// Returns a tuple of (successful families, errors with source names).
    pub async fn collect_all(&self) -> (Vec<MetricFamily>, Vec<(String, SourceError)>) {
        let start = Instant::now();

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

        let results = join_all(futures).await;

        let mut families = Vec::new();
        let mut errors = Vec::new();

        for (name, result) in results {
            match result {
                Ok(family) => {
                    debug!(
                        source = %name,
                        metrics = family.metrics.len(),
                        "Collected metrics"
                    );
                    families.push(family);
                }
                Err(e) => {
                    warn!(source = %name, error = %e, "Failed to collect metrics");
                    errors.push((name, e));
                }
            }
        }

        let duration = start.elapsed();
        debug!(
            duration_ms = duration.as_millis(),
            successful = families.len(),
            failed = errors.len(),
            "Collection complete"
        );

        (families, errors)
    }

    /// Checks health of all sources in parallel.
    ///
    /// Returns a map of source name -> healthy status.
    pub async fn health_check_all(&self) -> Vec<(String, bool)> {
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

        join_all(futures).await
    }

    /// Returns true if all sources are healthy.
    pub async fn all_healthy(&self) -> bool {
        let results = self.health_check_all().await;
        results.iter().all(|(_, healthy)| *healthy)
    }
}

impl std::fmt::Debug for SourceRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceRegistry")
            .field("sources_count", &self.sources.len())
            .finish()
    }
}
