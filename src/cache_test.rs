//! Tests for metrics cache.

use super::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_cache_disabled_ttl_zero() {
    let cache = MetricsCache::new(0);
    assert!(!cache.is_enabled());

    // Set should be no-op when disabled
    cache.set("test".to_string()).await;
    assert!(cache.get().await.is_none());
}

#[tokio::test]
async fn test_cache_enabled_positive_ttl() {
    let cache = MetricsCache::new(60);
    assert!(cache.is_enabled());

    cache.set("test data".to_string()).await;
    assert_eq!(cache.get().await.as_deref(), Some(&"test data".to_string()));
}

#[tokio::test]
async fn test_cache_clear() {
    let cache = MetricsCache::new(60);
    cache.set("data".to_string()).await;
    assert!(cache.get().await.is_some());

    cache.clear().await;
    assert!(cache.get().await.is_none());
}

#[tokio::test]
async fn test_cache_expiration() {
    // Use 1 second TTL for fast test
    let cache = MetricsCache::new(1);
    cache.set("expires soon".to_string()).await;

    // Should be present immediately
    assert_eq!(
        cache.get().await.as_deref(),
        Some(&"expires soon".to_string())
    );

    // Wait for expiration
    sleep(Duration::from_secs(2)).await;

    // Should be expired now
    assert!(cache.get().await.is_none());
}

#[tokio::test]
async fn test_cache_update_resets_expiration() {
    let cache = MetricsCache::new(2);

    cache.set("first".to_string()).await;
    sleep(Duration::from_secs(1)).await;

    // Update before expiration
    cache.set("second".to_string()).await;
    sleep(Duration::from_secs(1)).await;

    // Should still be valid (timer reset)
    assert_eq!(cache.get().await.as_deref(), Some(&"second".to_string()));
}

#[tokio::test]
async fn test_cache_overwrite() {
    let cache = MetricsCache::new(60);

    cache.set("first".to_string()).await;
    assert_eq!(cache.get().await.as_deref(), Some(&"first".to_string()));

    cache.set("second".to_string()).await;
    assert_eq!(cache.get().await.as_deref(), Some(&"second".to_string()));
}

#[tokio::test]
async fn test_cache_empty_string() {
    let cache = MetricsCache::new(60);

    cache.set(String::new()).await;
    assert_eq!(cache.get().await.as_deref(), Some(&String::new()));
}

#[tokio::test]
async fn test_cache_large_data() {
    let cache = MetricsCache::new(60);

    // 1MB of data
    let large_data = "x".repeat(1_000_000);
    cache.set(large_data.clone()).await;
    assert_eq!(cache.get().await.as_deref(), Some(&large_data));
}

#[tokio::test]
async fn test_cache_concurrent_reads() {
    let cache = Arc::new(MetricsCache::new(60));
    cache.set("shared data".to_string()).await;

    let mut handles = vec![];
    for _ in 0..10 {
        let c = cache.clone();
        handles.push(tokio::spawn(async move { c.get().await }));
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert_eq!(result.as_deref(), Some(&"shared data".to_string()));
    }
}

#[tokio::test]
async fn test_cache_concurrent_writes() {
    let cache = Arc::new(MetricsCache::new(60));

    let mut handles = vec![];
    for i in 0..10 {
        let c = cache.clone();
        handles.push(tokio::spawn(async move {
            c.set(format!("data_{}", i)).await;
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Should have some value (last write wins)
    assert!(cache.get().await.is_some());
}

#[tokio::test]
async fn test_create_cache_helper() {
    let cache = create_cache(30);
    assert!(cache.is_enabled());

    let cache_disabled = create_cache(0);
    assert!(!cache_disabled.is_enabled());
}

#[tokio::test]
async fn test_cache_get_before_set() {
    let cache = MetricsCache::new(60);
    assert!(cache.get().await.is_none());
}

#[tokio::test]
async fn test_cache_multiple_clears() {
    let cache = MetricsCache::new(60);

    // Clear empty cache
    cache.clear().await;
    assert!(cache.get().await.is_none());

    // Set and clear
    cache.set("data".to_string()).await;
    cache.clear().await;
    cache.clear().await; // Double clear
    assert!(cache.get().await.is_none());
}
