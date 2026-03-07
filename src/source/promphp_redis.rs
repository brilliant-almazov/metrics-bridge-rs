//! promphp Redis source implementation.
//!
//! Reads metrics from Redis stored by promphp/prometheus_client_php.

use super::{Source, SourceError, SourceResult};
use crate::config::{LabelFormat, SourceConfig};
use crate::metrics::{parse_promphp_metric, MetricFamily};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

/// Redis client abstraction for testing.
#[async_trait]
pub trait RedisClient: Send + Sync + std::fmt::Debug {
    /// Get all members of a set.
    async fn smembers(&self, key: &str) -> SourceResult<Vec<String>>;

    /// Get all fields and values of a hash.
    async fn hgetall(&self, key: &str) -> SourceResult<HashMap<String, String>>;

    /// Check if connection is alive.
    async fn ping(&self) -> bool;
}

/// Deadpool Redis client implementation.
#[derive(Debug)]
pub struct DeadpoolRedisClient {
    pool: deadpool_redis::Pool,
}

impl DeadpoolRedisClient {
    pub fn new(redis_url: &str) -> Result<Self, SourceError> {
        use deadpool_redis::{Config as PoolConfig, Runtime};

        let pool_config = PoolConfig::from_url(redis_url);
        let pool = pool_config
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| SourceError::Connection(e.to_string()))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl RedisClient for DeadpoolRedisClient {
    async fn smembers(&self, key: &str) -> SourceResult<Vec<String>> {
        use redis::AsyncCommands;

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| SourceError::Connection(e.to_string()))?;

        let keys: Vec<String> = conn
            .smembers(key)
            .await
            .map_err(|e| SourceError::Redis(e.to_string()))?;

        Ok(keys)
    }

    async fn hgetall(&self, key: &str) -> SourceResult<HashMap<String, String>> {
        use redis::AsyncCommands;

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| SourceError::Connection(e.to_string()))?;

        let data: HashMap<String, String> = conn
            .hgetall(key)
            .await
            .map_err(|e| SourceError::Redis(e.to_string()))?;

        Ok(data)
    }

    async fn ping(&self) -> bool {
        match self.pool.get().await {
            Ok(mut conn) => {
                let result: Result<String, _> = redis::cmd("PING").query_async(&mut *conn).await;
                result.is_ok()
            }
            Err(_) => false,
        }
    }
}

/// Source that reads promphp metrics from Redis.
#[derive(Debug)]
pub struct PromphpRedisSource<C: RedisClient = DeadpoolRedisClient> {
    name: String,
    prefix: String,
    extra_labels: HashMap<String, String>,
    label_format: LabelFormat,
    client: Arc<C>,
}

impl PromphpRedisSource<DeadpoolRedisClient> {
    /// Creates a new promphp Redis source with deadpool client.
    pub fn new(config: &SourceConfig) -> Result<Self, SourceError> {
        let client = DeadpoolRedisClient::new(&config.redis_url)?;

        Ok(Self {
            name: config.name.clone(),
            prefix: config.prefix.clone(),
            extra_labels: config.labels.clone(),
            label_format: config.label_format,
            client: Arc::new(client),
        })
    }
}

impl<C: RedisClient> PromphpRedisSource<C> {
    /// Creates a new promphp Redis source with custom client.
    pub fn with_client(
        name: String,
        prefix: String,
        extra_labels: HashMap<String, String>,
        client: Arc<C>,
    ) -> Self {
        Self {
            name,
            prefix,
            extra_labels,
            label_format: LabelFormat::Auto,
            client,
        }
    }

    /// Creates a new promphp Redis source with custom client and label format.
    pub fn with_client_and_format(
        name: String,
        prefix: String,
        extra_labels: HashMap<String, String>,
        label_format: LabelFormat,
        client: Arc<C>,
    ) -> Self {
        Self {
            name,
            prefix,
            extra_labels,
            label_format,
            client,
        }
    }

    /// Gets all metric keys for a given type (counter, gauge, histogram, summary).
    async fn get_metric_keys(&self, metric_type: &str) -> SourceResult<Vec<String>> {
        let set_key = format!("{}{}_METRIC_KEYS", self.prefix, metric_type);
        self.client.smembers(&set_key).await
    }

    /// Gets metric data from a Redis hash.
    async fn get_metric_data(&self, key: &str) -> SourceResult<HashMap<String, String>> {
        self.client.hgetall(key).await
    }
}

#[async_trait]
impl<C: RedisClient + 'static> Source for PromphpRedisSource<C> {
    fn name(&self) -> &str {
        &self.name
    }

    async fn collect(&self) -> SourceResult<MetricFamily> {
        let mut family = MetricFamily::new(&self.name).with_labels(self.extra_labels.clone());

        // Collect all metric types
        for metric_type in &["counter", "gauge", "histogram", "summary"] {
            let keys = self.get_metric_keys(metric_type).await?;

            debug!(
                source = %self.name,
                metric_type = %metric_type,
                count = keys.len(),
                "Found metric keys"
            );

            for hash_key in keys {
                let data = self.get_metric_data(&hash_key).await?;

                if data.is_empty() {
                    debug!(key = %hash_key, "Empty metric hash, skipping");
                    continue;
                }

                match parse_promphp_metric(data, self.label_format) {
                    Ok(metric) => {
                        family.add_metric(metric);
                    }
                    Err(e) => {
                        warn!(
                            source = %self.name,
                            key = %hash_key,
                            error = %e,
                            "Failed to parse metric"
                        );
                    }
                }
            }
        }

        Ok(family)
    }

    async fn health_check(&self) -> bool {
        self.client.ping().await
    }
}

#[cfg(test)]
#[path = "promphp_redis_test.rs"]
mod tests;
