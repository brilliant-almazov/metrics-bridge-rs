//! Metrics cache abstraction.
//!
//! Provides a unified caching layer that works identically whether
//! caching is enabled (TTL > 0) or disabled (TTL = 0).

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cached data with timestamp.
#[derive(Clone)]
struct CacheEntry {
    data: String,
    created_at: Instant,
}

/// Metrics cache with configurable TTL.
///
/// When TTL = 0, caching is disabled and `get()` always returns None.
/// When TTL > 0, data is cached for the specified duration.
pub struct MetricsCache {
    ttl: Duration,
    entry: RwLock<Option<CacheEntry>>,
}

impl MetricsCache {
    /// Creates a new cache with the given TTL in seconds.
    /// TTL of 0 means caching is disabled.
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            ttl: Duration::from_secs(ttl_seconds),
            entry: RwLock::new(None),
        }
    }

    /// Returns true if caching is enabled (TTL > 0).
    pub fn is_enabled(&self) -> bool {
        !self.ttl.is_zero()
    }

    /// Gets cached data if valid, None otherwise.
    pub async fn get(&self) -> Option<String> {
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
    pub async fn set(&self, data: String) {
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

/// Creates a shared cache instance.
pub fn create_cache(ttl_seconds: u64) -> Arc<MetricsCache> {
    Arc::new(MetricsCache::new(ttl_seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_disabled() {
        let cache = MetricsCache::new(0);
        assert!(!cache.is_enabled());

        cache.set("test".to_string()).await;
        assert!(cache.get().await.is_none());
    }

    #[tokio::test]
    async fn test_cache_enabled() {
        let cache = MetricsCache::new(60);
        assert!(cache.is_enabled());

        cache.set("test data".to_string()).await;
        assert_eq!(cache.get().await, Some("test data".to_string()));
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = MetricsCache::new(60);
        cache.set("data".to_string()).await;
        cache.clear().await;
        assert!(cache.get().await.is_none());
    }
}
