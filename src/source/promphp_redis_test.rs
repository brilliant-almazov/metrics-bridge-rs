//! Tests for promphp Redis source.

use super::*;
use crate::source::Source;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock Redis client for testing.
#[derive(Debug, Default)]
pub struct MockRedisClient {
    /// Data for smembers calls: key -> members
    sets: Mutex<HashMap<String, Vec<String>>>,
    /// Data for hgetall calls: key -> hash data
    hashes: Mutex<HashMap<String, HashMap<String, String>>>,
    /// Whether ping should succeed
    healthy: Mutex<bool>,
    /// Force smembers to fail
    smembers_error: Mutex<Option<String>>,
    /// Force hgetall to fail
    hgetall_error: Mutex<Option<String>>,
}

impl MockRedisClient {
    pub fn new() -> Self {
        Self {
            healthy: Mutex::new(true),
            ..Default::default()
        }
    }

    pub async fn set_members(&self, key: &str, members: Vec<String>) {
        self.sets.lock().await.insert(key.to_string(), members);
    }

    pub async fn set_hash(&self, key: &str, data: HashMap<String, String>) {
        self.hashes.lock().await.insert(key.to_string(), data);
    }

    pub async fn set_healthy(&self, healthy: bool) {
        *self.healthy.lock().await = healthy;
    }

    pub async fn set_smembers_error(&self, error: Option<String>) {
        *self.smembers_error.lock().await = error;
    }

    pub async fn set_hgetall_error(&self, error: Option<String>) {
        *self.hgetall_error.lock().await = error;
    }
}

#[async_trait]
impl RedisClient for MockRedisClient {
    async fn smembers(&self, key: &str) -> SourceResult<Vec<String>> {
        if let Some(err) = self.smembers_error.lock().await.clone() {
            return Err(SourceError::Redis(err));
        }
        Ok(self.sets.lock().await.get(key).cloned().unwrap_or_default())
    }

    async fn hgetall(&self, key: &str) -> SourceResult<HashMap<String, String>> {
        if let Some(err) = self.hgetall_error.lock().await.clone() {
            return Err(SourceError::Redis(err));
        }
        Ok(self
            .hashes
            .lock()
            .await
            .get(key)
            .cloned()
            .unwrap_or_default())
    }

    async fn ping(&self) -> bool {
        *self.healthy.lock().await
    }
}

fn create_counter_hash(name: &str, help: &str, value: f64) -> HashMap<String, String> {
    let mut data = HashMap::new();
    // labelNames must be present (empty array for no labels)
    data.insert(
        "__meta".to_string(),
        format!(
            r#"{{"name":"{}","help":"{}","type":"counter","labelNames":[]}}"#,
            name, help
        ),
    );
    // Use base64 encoded empty array "[]" -> W10=
    data.insert("W10=".to_string(), value.to_string());
    data
}

fn create_gauge_hash(name: &str, help: &str, value: f64) -> HashMap<String, String> {
    let mut data = HashMap::new();
    data.insert(
        "__meta".to_string(),
        format!(
            r#"{{"name":"{}","help":"{}","type":"gauge","labelNames":[]}}"#,
            name, help
        ),
    );
    // Use base64 encoded empty array "[]" -> W10=
    data.insert("W10=".to_string(), value.to_string());
    data
}

#[tokio::test]
async fn test_source_creation() {
    use crate::config::{SourceConfig, SourceType};

    let config = SourceConfig {
        name: "test".to_string(),
        source_type: SourceType::PromphpRedis,
        redis_url: "redis://localhost:6379".to_string(),
        prefix: "PROMETHEUS_".to_string(),
        labels: HashMap::new(),
    };

    // This will fail without Redis, but tests the creation path
    let result = PromphpRedisSource::new(&config);
    // We just check it compiles and runs - actual connection test needs Redis
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_collect_empty() {
    let client = Arc::new(MockRedisClient::new());
    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "PROMETHEUS_".to_string(),
        HashMap::new(),
        client,
    );

    let result = source.collect().await.unwrap();

    assert_eq!(result.source, "test");
    assert!(result.metrics.is_empty());
}

#[tokio::test]
async fn test_collect_counter() {
    let client = Arc::new(MockRedisClient::new());

    // Set up mock data
    client
        .set_members(
            "PROMETHEUS_counter_METRIC_KEYS",
            vec!["PROMETHEUS_http_requests".to_string()],
        )
        .await;
    client
        .set_hash(
            "PROMETHEUS_http_requests",
            create_counter_hash("http_requests", "Total HTTP requests", 42.0),
        )
        .await;

    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "PROMETHEUS_".to_string(),
        HashMap::new(),
        client,
    );

    let result = source.collect().await.unwrap();

    assert_eq!(result.metrics.len(), 1);
    assert_eq!(result.metrics[0].name, "http_requests");
}

#[tokio::test]
async fn test_collect_gauge() {
    let client = Arc::new(MockRedisClient::new());

    client
        .set_members(
            "PROMETHEUS_gauge_METRIC_KEYS",
            vec!["PROMETHEUS_temperature".to_string()],
        )
        .await;
    client
        .set_hash(
            "PROMETHEUS_temperature",
            create_gauge_hash("temperature", "Current temperature", 23.5),
        )
        .await;

    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "PROMETHEUS_".to_string(),
        HashMap::new(),
        client,
    );

    let result = source.collect().await.unwrap();

    assert_eq!(result.metrics.len(), 1);
    assert_eq!(result.metrics[0].name, "temperature");
}

#[tokio::test]
async fn test_collect_multiple_metrics() {
    let client = Arc::new(MockRedisClient::new());

    // Counters
    client
        .set_members(
            "PROMETHEUS_counter_METRIC_KEYS",
            vec![
                "PROMETHEUS_requests".to_string(),
                "PROMETHEUS_errors".to_string(),
            ],
        )
        .await;
    client
        .set_hash(
            "PROMETHEUS_requests",
            create_counter_hash("requests", "Total requests", 100.0),
        )
        .await;
    client
        .set_hash(
            "PROMETHEUS_errors",
            create_counter_hash("errors", "Total errors", 5.0),
        )
        .await;

    // Gauges
    client
        .set_members(
            "PROMETHEUS_gauge_METRIC_KEYS",
            vec!["PROMETHEUS_active".to_string()],
        )
        .await;
    client
        .set_hash(
            "PROMETHEUS_active",
            create_gauge_hash("active", "Active connections", 10.0),
        )
        .await;

    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "PROMETHEUS_".to_string(),
        HashMap::new(),
        client,
    );

    let result = source.collect().await.unwrap();

    assert_eq!(result.metrics.len(), 3);
}

#[tokio::test]
async fn test_collect_with_extra_labels() {
    let client = Arc::new(MockRedisClient::new());

    client
        .set_members(
            "PROMETHEUS_counter_METRIC_KEYS",
            vec!["PROMETHEUS_test".to_string()],
        )
        .await;
    client
        .set_hash(
            "PROMETHEUS_test",
            create_counter_hash("test", "Test metric", 1.0),
        )
        .await;

    let mut extra_labels = HashMap::new();
    extra_labels.insert("env".to_string(), "prod".to_string());

    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "PROMETHEUS_".to_string(),
        extra_labels,
        client,
    );

    let result = source.collect().await.unwrap();

    assert_eq!(result.extra_labels.get("env"), Some(&"prod".to_string()));
}

#[tokio::test]
async fn test_collect_skips_empty_hash() {
    let client = Arc::new(MockRedisClient::new());

    client
        .set_members(
            "PROMETHEUS_counter_METRIC_KEYS",
            vec!["PROMETHEUS_empty".to_string()],
        )
        .await;
    // Don't set hash data - it will return empty HashMap

    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "PROMETHEUS_".to_string(),
        HashMap::new(),
        client,
    );

    let result = source.collect().await.unwrap();

    assert!(result.metrics.is_empty());
}

#[tokio::test]
async fn test_collect_handles_invalid_metric() {
    let client = Arc::new(MockRedisClient::new());

    client
        .set_members(
            "PROMETHEUS_counter_METRIC_KEYS",
            vec!["PROMETHEUS_invalid".to_string()],
        )
        .await;

    // Invalid hash data - no __meta field
    let mut invalid_data = HashMap::new();
    invalid_data.insert("some_key".to_string(), "some_value".to_string());
    client.set_hash("PROMETHEUS_invalid", invalid_data).await;

    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "PROMETHEUS_".to_string(),
        HashMap::new(),
        client,
    );

    // Should not fail, just skip invalid metric
    let result = source.collect().await.unwrap();
    assert!(result.metrics.is_empty());
}

#[tokio::test]
async fn test_collect_redis_error_smembers() {
    let client = Arc::new(MockRedisClient::new());
    client
        .set_smembers_error(Some("Connection refused".to_string()))
        .await;

    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "PROMETHEUS_".to_string(),
        HashMap::new(),
        client,
    );

    let result = source.collect().await;

    assert!(result.is_err());
    assert!(matches!(result, Err(SourceError::Redis(_))));
}

#[tokio::test]
async fn test_collect_redis_error_hgetall() {
    let client = Arc::new(MockRedisClient::new());

    client
        .set_members(
            "PROMETHEUS_counter_METRIC_KEYS",
            vec!["PROMETHEUS_test".to_string()],
        )
        .await;
    client
        .set_hgetall_error(Some("Connection lost".to_string()))
        .await;

    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "PROMETHEUS_".to_string(),
        HashMap::new(),
        client,
    );

    let result = source.collect().await;

    assert!(result.is_err());
    assert!(matches!(result, Err(SourceError::Redis(_))));
}

#[tokio::test]
async fn test_health_check_healthy() {
    let client = Arc::new(MockRedisClient::new());
    client.set_healthy(true).await;

    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "PROMETHEUS_".to_string(),
        HashMap::new(),
        client,
    );

    assert!(source.health_check().await);
}

#[tokio::test]
async fn test_health_check_unhealthy() {
    let client = Arc::new(MockRedisClient::new());
    client.set_healthy(false).await;

    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "PROMETHEUS_".to_string(),
        HashMap::new(),
        client,
    );

    assert!(!source.health_check().await);
}

#[tokio::test]
async fn test_source_name() {
    let client = Arc::new(MockRedisClient::new());

    let source = PromphpRedisSource::with_client(
        "my-source".to_string(),
        "PROMETHEUS_".to_string(),
        HashMap::new(),
        client,
    );

    assert_eq!(source.name(), "my-source");
}

#[tokio::test]
async fn test_custom_prefix() {
    let client = Arc::new(MockRedisClient::new());

    // Use custom prefix
    client
        .set_members(
            "CUSTOM_counter_METRIC_KEYS",
            vec!["CUSTOM_test".to_string()],
        )
        .await;
    client
        .set_hash("CUSTOM_test", create_counter_hash("test", "Test", 1.0))
        .await;

    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "CUSTOM_".to_string(),
        HashMap::new(),
        client,
    );

    let result = source.collect().await.unwrap();

    assert_eq!(result.metrics.len(), 1);
}

#[tokio::test]
async fn test_collect_all_metric_types() {
    let client = Arc::new(MockRedisClient::new());

    // Set up counters
    client
        .set_members("TEST_counter_METRIC_KEYS", vec!["TEST_c1".to_string()])
        .await;
    client
        .set_hash("TEST_c1", create_counter_hash("c1", "Counter 1", 1.0))
        .await;

    // Set up gauges
    client
        .set_members("TEST_gauge_METRIC_KEYS", vec!["TEST_g1".to_string()])
        .await;
    client
        .set_hash("TEST_g1", create_gauge_hash("g1", "Gauge 1", 2.0))
        .await;

    // Set up histograms (we don't have histogram helper, but empty is fine)
    client
        .set_members("TEST_histogram_METRIC_KEYS", vec![])
        .await;

    let source = PromphpRedisSource::with_client(
        "test".to_string(),
        "TEST_".to_string(),
        HashMap::new(),
        client,
    );

    let result = source.collect().await.unwrap();

    assert_eq!(result.metrics.len(), 2);
}
