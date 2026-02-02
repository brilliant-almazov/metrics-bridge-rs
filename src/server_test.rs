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
            cache_ttl_seconds: None,
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
            cache_ttl_seconds: None,
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
            cache_ttl_seconds: None,
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
            cache_ttl_seconds: None,
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

#[test]
fn test_cached_metrics() {
    let cached = CachedMetrics {
        data: "test data".to_string(),
        timestamp: std::time::Instant::now(),
    };

    assert_eq!(cached.data, "test data");
}
