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

// Cache tests are in cache.rs
