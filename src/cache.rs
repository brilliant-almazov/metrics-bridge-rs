//! Metrics cache abstraction.
//!
//! Provides a unified caching layer that works identically whether
//! caching is enabled (TTL > 0) or disabled (TTL = 0).

use crate::metrics::MetricFamily;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cached data with timestamp.
#[derive(Clone)]
struct CacheEntry<T> {
    data: T,
    created_at: Instant,
}

/// Generic cache with configurable TTL.
///
/// When TTL = 0, caching is disabled and `get()` always returns None.
/// When TTL > 0, data is cached for the specified duration.
pub struct Cache<T> {
    ttl: Duration,
    entry: RwLock<Option<CacheEntry<T>>>,
}

impl<T: Clone> Cache<T> {
    /// Creates a new cache with the given TTL in seconds.
    /// TTL of 0 means caching is disabled.
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            ttl: Duration::from_secs(ttl_seconds),
            entry: RwLock::new(None),
        }
    }

    /// Returns the TTL in seconds.
    pub fn ttl_seconds(&self) -> u64 {
        self.ttl.as_secs()
    }

    /// Returns true if caching is enabled (TTL > 0).
    pub fn is_enabled(&self) -> bool {
        !self.ttl.is_zero()
    }

    /// Gets cached data if valid, None otherwise.
    pub async fn get(&self) -> Option<T> {
        if !self.is_enabled() {
            return None;
        }

        let entry = self.entry.read().await;
        entry.as_ref().and_then(|e| {
            if e.created_at.elapsed() < self.ttl {
                Some(e.data.clone())
            } else {
                None
            }
        })
    }

    /// Stores data in cache. No-op if caching is disabled.
    pub async fn set(&self, data: T) {
        if !self.is_enabled() {
            return;
        }

        let mut entry = self.entry.write().await;
        *entry = Some(CacheEntry {
            data,
            created_at: Instant::now(),
        });
    }

    /// Clears the cache.
    pub async fn clear(&self) {
        let mut entry = self.entry.write().await;
        *entry = None;
    }
}

/// Type alias for string cache (rendered metrics).
pub type MetricsCache = Cache<String>;

/// Type alias for metric family cache (per-source).
pub type MetricFamilyCache = Cache<MetricFamily>;

/// Creates a shared cache instance.
pub fn create_cache(ttl_seconds: u64) -> Arc<MetricsCache> {
    Arc::new(MetricsCache::new(ttl_seconds))
}

#[cfg(test)]
#[path = "cache_test.rs"]
mod tests;
