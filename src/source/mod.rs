//! Metric source abstraction.
//!
//! Defines the Source trait and provides implementations for different backends.

mod promphp_redis;
mod registry;

pub use promphp_redis::PromphpRedisSource;
pub use registry::SourceRegistry;

use crate::metrics::MetricFamily;
use async_trait::async_trait;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Timeout")]
    Timeout,

    #[error("Redis error: {0}")]
    Redis(String),
}

/// Result type for source operations.
pub type SourceResult<T> = Result<T, SourceError>;

/// A source of metrics that can be collected.
#[async_trait]
pub trait Source: Send + Sync + Debug {
    /// Returns the source name.
    fn name(&self) -> &str;

    /// Collects metrics from this source.
    async fn collect(&self) -> SourceResult<MetricFamily>;

    /// Checks if the source is healthy/reachable.
    async fn health_check(&self) -> bool;
}
