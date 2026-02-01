//! Tests for config module.

use super::*;

#[test]
fn test_expand_env_vars() {
    std::env::set_var("TEST_VAR", "test_value");
    let result = expand_env_vars("prefix_${TEST_VAR}_suffix").unwrap();
    assert_eq!(result, "prefix_test_value_suffix");
    std::env::remove_var("TEST_VAR");
}

#[test]
fn test_expand_env_vars_missing() {
    let result = expand_env_vars("${DEFINITELY_NOT_SET_VAR_12345}");
    assert!(result.is_err());
}

#[test]
fn test_auth_type_display() {
    assert_eq!(AuthType::None.to_string(), "none");
    assert_eq!(AuthType::Basic.to_string(), "basic");
    assert_eq!(AuthType::Bearer.to_string(), "bearer");
}

#[test]
fn test_source_type_display() {
    assert_eq!(SourceType::PromphpRedis.to_string(), "promphp-redis");
}

#[test]
fn test_is_ip_allowed_empty() {
    let config = Config {
        server: ServerConfig::default(),
        sources: vec![],
        allowed_networks: vec![],
    };
    assert!(config.is_ip_allowed("192.168.1.1".parse().unwrap()));
}

#[test]
fn test_is_ip_allowed_match() {
    let config = Config {
        server: ServerConfig::default(),
        sources: vec![],
        allowed_networks: vec!["10.0.0.0/8".parse().unwrap()],
    };
    assert!(config.is_ip_allowed("10.1.2.3".parse().unwrap()));
    assert!(!config.is_ip_allowed("192.168.1.1".parse().unwrap()));
}

#[test]
fn test_tls_config_is_enabled() {
    let disabled = TlsConfig::default();
    assert!(!disabled.is_enabled());

    let enabled = TlsConfig {
        cert: Some("/path/to/cert".to_string()),
        key: Some("/path/to/key".to_string()),
    };
    assert!(enabled.is_enabled());

    let partial = TlsConfig {
        cert: Some("/path/to/cert".to_string()),
        key: None,
    };
    assert!(!partial.is_enabled());
}
