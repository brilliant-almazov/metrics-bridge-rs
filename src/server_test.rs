//! Tests for HTTP server.

use super::*;
use crate::config::{AuthConfig, AuthType, Config, ServerConfig, TlsConfig};
use axum::http::Request;
use base64::{engine::general_purpose::STANDARD, Engine};

#[test]
fn test_check_basic_auth_valid() {
    let config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig {
                auth_type: AuthType::Basic,
                username: Some("admin".to_string()),
                password: Some("secret".to_string()),
                token: None,
            },
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    // Valid credentials
    let credentials = STANDARD.encode("admin:secret");
    let request = Request::builder()
        .header("Authorization", format!("Basic {}", credentials))
        .body(Body::empty())
        .unwrap();

    assert!(check_basic_auth(&request, &config));
}

#[test]
fn test_check_basic_auth_invalid() {
    let config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig {
                auth_type: AuthType::Basic,
                username: Some("admin".to_string()),
                password: Some("secret".to_string()),
                token: None,
            },
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    // Wrong password
    let credentials = STANDARD.encode("admin:wrong");
    let request = Request::builder()
        .header("Authorization", format!("Basic {}", credentials))
        .body(Body::empty())
        .unwrap();

    assert!(!check_basic_auth(&request, &config));
}

#[test]
fn test_check_basic_auth_no_header() {
    let config = Config {
        server: ServerConfig::default(),
        sources: vec![],
        allowed_networks: vec![],
    };

    let request = Request::builder().body(Body::empty()).unwrap();

    assert!(!check_basic_auth(&request, &config));
}

#[test]
fn test_check_bearer_auth_valid() {
    let config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig {
                auth_type: AuthType::Bearer,
                username: None,
                password: None,
                token: Some("my-secret-token".to_string()),
            },
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    let request = Request::builder()
        .header("Authorization", "Bearer my-secret-token")
        .body(Body::empty())
        .unwrap();

    assert!(check_bearer_auth(&request, &config));
}

#[test]
fn test_check_bearer_auth_invalid() {
    let config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig {
                auth_type: AuthType::Bearer,
                username: None,
                password: None,
                token: Some("my-secret-token".to_string()),
            },
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    let request = Request::builder()
        .header("Authorization", "Bearer wrong-token")
        .body(Body::empty())
        .unwrap();

    assert!(!check_bearer_auth(&request, &config));
}

#[test]
fn test_check_bearer_auth_no_header() {
    let config = Config {
        server: ServerConfig::default(),
        sources: vec![],
        allowed_networks: vec![],
    };

    let request = Request::builder().body(Body::empty()).unwrap();

    assert!(!check_bearer_auth(&request, &config));
}

// Additional auth edge case tests

#[test]
fn test_check_basic_auth_wrong_scheme() {
    let config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig {
                auth_type: AuthType::Basic,
                username: Some("admin".to_string()),
                password: Some("secret".to_string()),
                token: None,
            },
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    // Bearer token instead of Basic
    let request = Request::builder()
        .header("Authorization", "Bearer some-token")
        .body(Body::empty())
        .unwrap();

    assert!(!check_basic_auth(&request, &config));
}

#[test]
fn test_check_basic_auth_invalid_base64() {
    let config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig {
                auth_type: AuthType::Basic,
                username: Some("admin".to_string()),
                password: Some("secret".to_string()),
                token: None,
            },
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    // Invalid base64
    let request = Request::builder()
        .header("Authorization", "Basic not-valid-base64!!!")
        .body(Body::empty())
        .unwrap();

    assert!(!check_basic_auth(&request, &config));
}

#[test]
fn test_check_basic_auth_no_colon() {
    let config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig {
                auth_type: AuthType::Basic,
                username: Some("admin".to_string()),
                password: Some("secret".to_string()),
                token: None,
            },
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    // Base64 without colon separator
    let credentials = STANDARD.encode("nocolonseparator");
    let request = Request::builder()
        .header("Authorization", format!("Basic {}", credentials))
        .body(Body::empty())
        .unwrap();

    assert!(!check_basic_auth(&request, &config));
}

#[test]
fn test_check_basic_auth_wrong_username() {
    let config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig {
                auth_type: AuthType::Basic,
                username: Some("admin".to_string()),
                password: Some("secret".to_string()),
                token: None,
            },
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    let credentials = STANDARD.encode("wronguser:secret");
    let request = Request::builder()
        .header("Authorization", format!("Basic {}", credentials))
        .body(Body::empty())
        .unwrap();

    assert!(!check_basic_auth(&request, &config));
}

#[test]
fn test_check_basic_auth_empty_credentials() {
    let config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig {
                auth_type: AuthType::Basic,
                username: Some("".to_string()),
                password: Some("".to_string()),
                token: None,
            },
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    let credentials = STANDARD.encode(":");
    let request = Request::builder()
        .header("Authorization", format!("Basic {}", credentials))
        .body(Body::empty())
        .unwrap();

    assert!(check_basic_auth(&request, &config));
}

#[test]
fn test_check_basic_auth_password_with_colon() {
    let config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig {
                auth_type: AuthType::Basic,
                username: Some("admin".to_string()),
                password: Some("pass:word:with:colons".to_string()),
                token: None,
            },
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    let credentials = STANDARD.encode("admin:pass:word:with:colons");
    let request = Request::builder()
        .header("Authorization", format!("Basic {}", credentials))
        .body(Body::empty())
        .unwrap();

    assert!(check_basic_auth(&request, &config));
}

#[test]
fn test_check_bearer_auth_wrong_scheme() {
    let config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig {
                auth_type: AuthType::Bearer,
                username: None,
                password: None,
                token: Some("my-token".to_string()),
            },
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    // Basic instead of Bearer
    let credentials = STANDARD.encode("user:pass");
    let request = Request::builder()
        .header("Authorization", format!("Basic {}", credentials))
        .body(Body::empty())
        .unwrap();

    assert!(!check_bearer_auth(&request, &config));
}

#[test]
fn test_check_bearer_auth_empty_token_config() {
    let config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig {
                auth_type: AuthType::Bearer,
                username: None,
                password: None,
                token: None,
            },
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    // Empty token matches empty config
    let request = Request::builder()
        .header("Authorization", "Bearer ")
        .body(Body::empty())
        .unwrap();

    assert!(check_bearer_auth(&request, &config));
}

#[test]
fn test_app_state_new() {
    let _config = Config {
        server: ServerConfig {
            port: 9090,
            auth: AuthConfig::default(),
            allowed_ips: vec![],
            tls: TlsConfig::default(),
            cache_ttl_seconds: 30,
            gzip_level: Some(6),
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    // Note: This requires a valid SourceRegistry, which needs actual sources
    // For now we test that Config construction works
    // In production, you'd use SourceRegistry::from_config
}

#[tokio::test]
async fn test_health_handler() {
    let response = health_handler().await.into_response();
    assert_eq!(response.status(), StatusCode::OK);
}

// Integration tests for handlers

use crate::metrics::{Metric, MetricFamily, MetricType, Sample};
use crate::source::{Source, SourceError, SourceResult};
use std::collections::HashMap;

/// Mock source for integration testing.
#[derive(Debug)]
struct MockSource {
    name: String,
    should_fail: bool,
    healthy: bool,
}

impl MockSource {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            should_fail: false,
            healthy: true,
        }
    }

    fn failing(name: &str) -> Self {
        Self {
            name: name.to_string(),
            should_fail: true,
            healthy: false,
        }
    }
}

#[async_trait::async_trait]
impl Source for MockSource {
    fn name(&self) -> &str {
        &self.name
    }

    async fn collect(&self) -> SourceResult<MetricFamily> {
        if self.should_fail {
            return Err(SourceError::Connection("mock failure".to_string()));
        }

        let mut family = MetricFamily::new(&self.name);
        let mut metric = Metric::new(
            format!("{}_requests", self.name),
            "Mock metric",
            MetricType::Counter,
        );
        metric.add_sample(Sample::new(HashMap::new(), 42.0));
        family.add_metric(metric);

        Ok(family)
    }

    async fn health_check(&self) -> bool {
        self.healthy
    }
}

#[tokio::test]
async fn test_ready_handler_all_healthy() {
    use crate::source::SourceRegistry;

    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(MockSource::new("source1")),
        Arc::new(MockSource::new("source2")),
    ];

    let registry = SourceRegistry::with_sources(sources);
    let config = Config {
        server: ServerConfig::default(),
        sources: vec![],
        allowed_networks: vec![],
    };

    let state = Arc::new(AppState {
        config,
        registry,
    });

    let response = ready_handler(State(state)).await.into_response();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ready_handler_some_unhealthy() {
    use crate::source::SourceRegistry;

    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(MockSource::new("healthy")),
        Arc::new(MockSource::failing("unhealthy")),
    ];

    let registry = SourceRegistry::with_sources(sources);
    let config = Config {
        server: ServerConfig::default(),
        sources: vec![],
        allowed_networks: vec![],
    };

    let state = Arc::new(AppState {
        config,
        registry,
    });

    let response = ready_handler(State(state)).await.into_response();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_ready_handler_empty_registry() {
    use crate::source::SourceRegistry;

    let registry = SourceRegistry::new();
    let config = Config {
        server: ServerConfig::default(),
        sources: vec![],
        allowed_networks: vec![],
    };

    let state = Arc::new(AppState {
        config,
        registry,
    });

    let response = ready_handler(State(state)).await.into_response();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_metrics_handler_success() {
    use crate::source::SourceRegistry;

    let sources: Vec<Arc<dyn Source>> = vec![Arc::new(MockSource::new("test"))];

    let registry = SourceRegistry::with_sources(sources);
    let config = Config {
        server: ServerConfig::default(),
        sources: vec![],
        allowed_networks: vec![],
    };

    let state = Arc::new(AppState {
        config,
        registry,
    });

    let response = metrics_handler(State(state)).await.into_response();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_metrics_handler_with_cache() {
    use crate::source::SourceRegistry;

    let sources: Vec<Arc<dyn Source>> = vec![Arc::new(MockSource::new("cached"))];

    let registry = SourceRegistry::with_sources(sources);
    let config = Config {
        server: ServerConfig {
            cache_ttl_seconds: 60,
            ..Default::default()
        },
        sources: vec![],
        allowed_networks: vec![],
    };

    let state = Arc::new(AppState {
        config,
        registry,
    });

    // First request - populates cache
    let response1 = metrics_handler(State(state.clone())).await.into_response();
    assert_eq!(response1.status(), StatusCode::OK);

    // Second request - should use cache
    let response2 = metrics_handler(State(state)).await.into_response();
    assert_eq!(response2.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_metrics_handler_with_errors() {
    use crate::source::SourceRegistry;

    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(MockSource::new("ok")),
        Arc::new(MockSource::failing("fail")),
    ];

    let registry = SourceRegistry::with_sources(sources);
    let config = Config {
        server: ServerConfig::default(),
        sources: vec![],
        allowed_networks: vec![],
    };

    let state = Arc::new(AppState {
        config,
        registry,
    });

    // Should still return 200, just with fewer metrics
    let response = metrics_handler(State(state)).await.into_response();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_metrics_handler_empty_registry() {
    use crate::source::SourceRegistry;

    let registry = SourceRegistry::new();
    let config = Config {
        server: ServerConfig::default(),
        sources: vec![],
        allowed_networks: vec![],
    };

    let state = Arc::new(AppState {
        config,
        registry,
    });

    let response = metrics_handler(State(state)).await.into_response();
    assert_eq!(response.status(), StatusCode::OK);
}

// Integration tests using tower::ServiceExt
use tower::ServiceExt;

#[test]
fn test_gzip_level_to_quality() {
    use tower_http::CompressionLevel;

    // Fastest (1-3)
    assert!(matches!(gzip_level_to_quality(1), CompressionLevel::Fastest));
    assert!(matches!(gzip_level_to_quality(2), CompressionLevel::Fastest));
    assert!(matches!(gzip_level_to_quality(3), CompressionLevel::Fastest));

    // Default (4-6)
    assert!(matches!(gzip_level_to_quality(4), CompressionLevel::Default));
    assert!(matches!(gzip_level_to_quality(5), CompressionLevel::Default));
    assert!(matches!(gzip_level_to_quality(6), CompressionLevel::Default));

    // Best (7-9)
    assert!(matches!(gzip_level_to_quality(7), CompressionLevel::Best));
    assert!(matches!(gzip_level_to_quality(8), CompressionLevel::Best));
    assert!(matches!(gzip_level_to_quality(9), CompressionLevel::Best));

    // Out of range defaults to Default
    assert!(matches!(gzip_level_to_quality(0), CompressionLevel::Default));
    assert!(matches!(gzip_level_to_quality(10), CompressionLevel::Default));
}

#[tokio::test]
async fn test_build_router_health_endpoint() {
    use crate::source::SourceRegistry;

    let state = Arc::new(AppState {
        config: Config {
            server: ServerConfig::default(),
            sources: vec![],
            allowed_networks: vec![],
        },
        registry: SourceRegistry::new(),
    });

    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_build_router_metrics_endpoint() {
    use crate::source::SourceRegistry;
    use axum::extract::ConnectInfo;

    let sources: Vec<Arc<dyn Source>> = vec![Arc::new(MockSource::new("test"))];
    let state = Arc::new(AppState {
        config: Config {
            server: ServerConfig::default(),
            sources: vec![],
            allowed_networks: vec![],
        },
        registry: SourceRegistry::with_sources(sources),
    });

    let app = build_router(state);

    let mut request = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))));

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_build_router_ready_endpoint_healthy() {
    use crate::source::SourceRegistry;

    let sources: Vec<Arc<dyn Source>> = vec![Arc::new(MockSource::new("healthy"))];
    let state = Arc::new(AppState {
        config: Config {
            server: ServerConfig::default(),
            sources: vec![],
            allowed_networks: vec![],
        },
        registry: SourceRegistry::with_sources(sources),
    });

    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_build_router_ready_endpoint_unhealthy() {
    use crate::source::SourceRegistry;

    let sources: Vec<Arc<dyn Source>> = vec![Arc::new(MockSource::failing("unhealthy"))];
    let state = Arc::new(AppState {
        config: Config {
            server: ServerConfig::default(),
            sources: vec![],
            allowed_networks: vec![],
        },
        registry: SourceRegistry::with_sources(sources),
    });

    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}
