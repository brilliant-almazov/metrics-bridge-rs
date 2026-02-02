//! promphp Redis source implementation.
//!
//! Reads metrics from Redis stored by promphp/prometheus_client_php.

use super::{Source, SourceError, SourceResult};
use crate::config::SourceConfig;
use crate::metrics::{parse_promphp_metric, MetricFamily};
use async_trait::async_trait;
use deadpool_redis::{Config as PoolConfig, Pool, Runtime};
use redis::AsyncCommands;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Source that reads promphp metrics from Redis.
#[derive(Debug)]
pub struct PromphpRedisSource {
    name: String,
    prefix: String,
    extra_labels: HashMap<String, String>,
    pool: Pool,
}

impl PromphpRedisSource {
    /// Creates a new promphp Redis source.
    pub fn new(config: &SourceConfig) -> Result<Self, SourceError> {
        let pool_config = PoolConfig::from_url(&config.redis_url);
        let pool = pool_config
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| SourceError::Connection(e.to_string()))?;

        Ok(Self {
            name: config.name.clone(),
            prefix: config.prefix.clone(),
            extra_labels: config.labels.clone(),
            pool,
        })
    }

    /// Gets all metric keys for a given type (counter, gauge, histogram).
    /// Returns the full hash key names directly from the SET.
    async fn get_metric_keys(
        &self,
        conn: &mut deadpool_redis::Connection,
        metric_type: &str,
    ) -> SourceResult<Vec<String>> {
        // promphp stores keys as: {prefix}{type}_METRIC_KEYS
        // e.g., PROMETHEUS_counter_METRIC_KEYS
        let set_key = format!("{}{}_METRIC_KEYS", self.prefix, metric_type);
        let keys: Vec<String> = conn
            .smembers(&set_key)
            .await
            .map_err(|e| SourceError::Redis(e.to_string()))?;
        Ok(keys)
    }

    /// Gets metric data from a Redis hash.
    async fn get_metric_data(
        &self,
        conn: &mut deadpool_redis::Connection,
        key: &str,
    ) -> SourceResult<HashMap<String, String>> {
        let data: HashMap<String, String> = conn
            .hgetall(key)
            .await
            .map_err(|e| SourceError::Redis(e.to_string()))?;
        Ok(data)
    }
}

#[async_trait]
impl Source for PromphpRedisSource {
    fn name(&self) -> &str {
        &self.name
    }

    async fn collect(&self) -> SourceResult<MetricFamily> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| SourceError::Connection(e.to_string()))?;

        let mut family = MetricFamily::new(&self.name).with_labels(self.extra_labels.clone());

        // Collect all metric types
        for metric_type in &["counter", "gauge", "histogram"] {
            let keys = self.get_metric_keys(&mut conn, metric_type).await?;

            debug!(
                source = %self.name,
                metric_type = %metric_type,
                count = keys.len(),
                "Found metric keys"
            );

            for hash_key in keys {
                // The SET contains full hash key names, use them directly
                let data = self.get_metric_data(&mut conn, &hash_key).await?;

                if data.is_empty() {
                    debug!(key = %hash_key, "Empty metric hash, skipping");
                    continue;
                }

                match parse_promphp_metric(data) {
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
        match self.pool.get().await {
            Ok(mut conn) => {
                let result: Result<String, _> = redis::cmd("PING").query_async(&mut *conn).await;
                result.is_ok()
            }
            Err(_) => false,
        }
    }
}

#[cfg(test)]
#[path = "promphp_redis_test.rs"]
mod tests;
