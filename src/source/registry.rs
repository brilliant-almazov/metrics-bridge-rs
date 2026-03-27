//! Source registry for managing multiple metric sources.

use super::{PromphpRedisSource, Source, SourceError};
use crate::cache::MetricFamilyCache;
use crate::config::{Config, SourceType};
use crate::metrics::MetricFamily;
use futures::future::join_all;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

/// A source entry with optional per-source caching.
struct SourceEntry {
    source: Arc<dyn Source>,
    cache: MetricFamilyCache,
}

/// Registry holding all configured metric sources.
pub struct SourceRegistry {
    entries: Vec<SourceEntry>,
}

impl SourceRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    /// Creates a registry with the given sources (no caching).
    pub fn with_sources(sources: Vec<Arc<dyn Source>>) -> Self {
        let entries = sources
            .into_iter()
            .map(|source| SourceEntry {
                source,
                cache: MetricFamilyCache::new(0), // No caching
            })
            .collect();
        Self { entries }
    }

    /// Creates a registry from sources with custom TTLs.
    pub fn with_sources_and_ttls(sources: Vec<(Arc<dyn Source>, u64)>) -> Self {
        let entries = sources
            .into_iter()
            .map(|(source, ttl)| SourceEntry {
                source,
                cache: MetricFamilyCache::new(ttl),
            })
            .collect();
        Self { entries }
    }

    /// Creates a new registry from configuration with per-source caching.
    pub fn from_config(config: &Config) -> Result<Self, SourceError> {
        let mut entries = Vec::new();

        for source_config in &config.sources {
            let cache_ttl = source_config.cache_ttl_seconds;
            info!(
                name = %source_config.name,
                source_type = %source_config.source_type,
                cache_ttl_seconds = cache_ttl,
                "Initializing source"
            );

            let source: Arc<dyn Source> = match source_config.source_type {
                SourceType::PromphpRedis => Arc::new(PromphpRedisSource::new(source_config)?),
            };

            entries.push(SourceEntry {
                source,
                cache: MetricFamilyCache::new(cache_ttl),
            });
        }

        Ok(Self { entries })
    }

    /// Returns the number of registered sources.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if no sources are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Collects metrics from all sources in parallel with per-source caching.
    ///
    /// Returns a tuple of (successful families, errors with source names).
    /// Each source uses its own cache with individual TTL.
    /// Families are returned behind `Arc` to avoid deep cloning.
    pub async fn collect_all(&self) -> (Vec<Arc<MetricFamily>>, Vec<(String, SourceError)>) {
        let start = Instant::now();

        let futures: Vec<_> = self
            .entries
            .iter()
            .map(|entry| {
                let source = Arc::clone(&entry.source);
                let cache = &entry.cache;
                async move {
                    let name = source.name().to_string();

                    // Check cache first
                    if let Some(cached) = cache.get().await {
                        debug!(source = %name, "Using cached metrics");
                        return (name, Ok(cached), true);
                    }

                    // Cache miss - collect from source
                    let result = source.collect().await;

                    // Wrap in Arc and store in cache if successful
                    let result = match result {
                        Ok(family) => {
                            let arc_family = Arc::new(family);
                            cache.set_arc(Arc::clone(&arc_family)).await;
                            Ok(arc_family)
                        }
                        Err(e) => Err(e),
                    };

                    (name, result, false)
                }
            })
            .collect();

        let results = join_all(futures).await;

        let mut families = Vec::new();
        let mut errors = Vec::new();
        let mut cache_hits = 0;

        for (name, result, was_cached) in results {
            if was_cached {
                cache_hits += 1;
            }
            match result {
                Ok(family) => {
                    debug!(
                        source = %name,
                        metrics = family.metrics.len(),
                        cached = was_cached,
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
            cache_hits = cache_hits,
            "Collection complete"
        );

        (families, errors)
    }

    /// Checks health of all sources in parallel.
    ///
    /// Returns a map of source name -> healthy status.
    pub async fn health_check_all(&self) -> Vec<(String, bool)> {
        let futures: Vec<_> = self
            .entries
            .iter()
            .map(|entry| {
                let source = Arc::clone(&entry.source);
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

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SourceRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceRegistry")
            .field("sources_count", &self.entries.len())
            .finish()
    }
}

#[cfg(test)]
#[path = "registry_test.rs"]
mod tests;
