//! metrics-bridge - Fast Prometheus metrics exporter for promphp Redis storage.
//!
//! Reads metrics written by PHP's promphp/prometheus_client_php from Redis
//! and exposes them in Prometheus format with sub-millisecond latency.

pub mod cache;
pub mod config;
pub mod metrics;
pub mod self_metrics;
pub mod server;
pub mod source;

pub use config::Config;
pub use server::start_server;
pub use source::SourceRegistry;
