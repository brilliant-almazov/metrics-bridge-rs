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
fn test_expand_env_vars_no_vars() {
    let result = expand_env_vars("no variables here").unwrap();
    assert_eq!(result, "no variables here");
}

#[test]
fn test_expand_env_vars_multiple() {
    std::env::set_var("VAR_A", "first");
    std::env::set_var("VAR_B", "second");
    let result = expand_env_vars("${VAR_A} and ${VAR_B}").unwrap();
    assert_eq!(result, "first and second");
    std::env::remove_var("VAR_A");
    std::env::remove_var("VAR_B");
}

#[test]
fn test_expand_env_vars_empty_value() {
    std::env::set_var("EMPTY_VAR", "");
    let result = expand_env_vars("prefix${EMPTY_VAR}suffix").unwrap();
    assert_eq!(result, "prefixsuffix");
    std::env::remove_var("EMPTY_VAR");
}

#[test]
fn test_expand_env_vars_dollar_without_brace() {
    let result = expand_env_vars("$not_a_var").unwrap();
    assert_eq!(result, "$not_a_var");
}

#[test]
fn test_parse_allowed_ips_single_ip() {
    use std::net::IpAddr;
    let ips = vec!["192.168.1.1".to_string()];
    let networks = Config::parse_allowed_ips(&ips).unwrap();
    assert_eq!(networks.len(), 1);
    let ip: IpAddr = "192.168.1.1".parse().unwrap();
    assert!(networks[0].contains(&ip));
    let ip2: IpAddr = "192.168.1.2".parse().unwrap();
    assert!(!networks[0].contains(&ip2));
}

#[test]
fn test_parse_allowed_ips_cidr() {
    use std::net::IpAddr;
    let ips = vec!["10.0.0.0/8".to_string()];
    let networks = Config::parse_allowed_ips(&ips).unwrap();
    assert_eq!(networks.len(), 1);
    let ip1: IpAddr = "10.255.255.255".parse().unwrap();
    assert!(networks[0].contains(&ip1));
    let ip2: IpAddr = "11.0.0.0".parse().unwrap();
    assert!(!networks[0].contains(&ip2));
}

#[test]
fn test_parse_allowed_ips_ipv6() {
    use std::net::IpAddr;
    let ips = vec!["::1".to_string()];
    let networks = Config::parse_allowed_ips(&ips).unwrap();
    assert_eq!(networks.len(), 1);
    let ip: IpAddr = "::1".parse().unwrap();
    assert!(networks[0].contains(&ip));
}

#[test]
fn test_parse_allowed_ips_invalid() {
    let ips = vec!["not-an-ip".to_string()];
    let result = Config::parse_allowed_ips(&ips);
    assert!(result.is_err());
}

#[test]
fn test_parse_allowed_ips_invalid_cidr() {
    let ips = vec!["192.168.1.0/99".to_string()];
    let result = Config::parse_allowed_ips(&ips);
    assert!(result.is_err());
}

#[test]
fn test_validate_auth_none() {
    let auth = AuthConfig {
        auth_type: AuthType::None,
        username: None,
        password: None,
        token: None,
    };
    assert!(Config::validate_auth(&auth).is_ok());
}

#[test]
fn test_validate_auth_basic_valid() {
    let auth = AuthConfig {
        auth_type: AuthType::Basic,
        username: Some("user".to_string()),
        password: Some("pass".to_string()),
        token: None,
    };
    assert!(Config::validate_auth(&auth).is_ok());
}

#[test]
fn test_validate_auth_basic_missing_username() {
    let auth = AuthConfig {
        auth_type: AuthType::Basic,
        username: None,
        password: Some("pass".to_string()),
        token: None,
    };
    assert!(Config::validate_auth(&auth).is_err());
}

#[test]
fn test_validate_auth_basic_missing_password() {
    let auth = AuthConfig {
        auth_type: AuthType::Basic,
        username: Some("user".to_string()),
        password: None,
        token: None,
    };
    assert!(Config::validate_auth(&auth).is_err());
}

#[test]
fn test_validate_auth_bearer_valid() {
    let auth = AuthConfig {
        auth_type: AuthType::Bearer,
        username: None,
        password: None,
        token: Some("secret-token".to_string()),
    };
    assert!(Config::validate_auth(&auth).is_ok());
}

#[test]
fn test_validate_auth_bearer_missing_token() {
    let auth = AuthConfig {
        auth_type: AuthType::Bearer,
        username: None,
        password: None,
        token: None,
    };
    assert!(Config::validate_auth(&auth).is_err());
}

#[test]
fn test_is_ip_allowed_ipv6() {
    let config = Config {
        server: ServerConfig::default(),
        sources: vec![],
        allowed_networks: vec!["::1/128".parse().unwrap()],
    };
    assert!(config.is_ip_allowed("::1".parse().unwrap()));
    assert!(!config.is_ip_allowed("::2".parse().unwrap()));
}

#[test]
fn test_is_ip_allowed_multiple_networks() {
    let config = Config {
        server: ServerConfig::default(),
        sources: vec![],
        allowed_networks: vec![
            "10.0.0.0/8".parse().unwrap(),
            "192.168.0.0/16".parse().unwrap(),
        ],
    };
    assert!(config.is_ip_allowed("10.1.2.3".parse().unwrap()));
    assert!(config.is_ip_allowed("192.168.1.1".parse().unwrap()));
    assert!(!config.is_ip_allowed("172.16.0.1".parse().unwrap()));
}

#[test]
fn test_server_config_default() {
    let config = ServerConfig::default();
    assert_eq!(config.port, 9090);
    assert_eq!(config.cache_ttl_seconds, 0);
    assert!(config.gzip_level.is_none());
    assert!(config.allowed_ips.is_empty());
}

#[test]
fn test_auth_config_default() {
    let auth = AuthConfig::default();
    assert_eq!(auth.auth_type, AuthType::None);
    assert!(auth.username.is_none());
    assert!(auth.password.is_none());
    assert!(auth.token.is_none());
}

#[test]
fn test_config_error_display() {
    let e = ConfigError::MissingValue("test".to_string());
    assert!(e.to_string().contains("test"));

    let e = ConfigError::FileError("file error".to_string());
    assert!(e.to_string().contains("file error"));

    let e = ConfigError::YamlError("yaml error".to_string());
    assert!(e.to_string().contains("yaml error"));

    let e = ConfigError::Base64Error("base64 error".to_string());
    assert!(e.to_string().contains("base64 error"));

    let e = ConfigError::InvalidValue("invalid".to_string());
    assert!(e.to_string().contains("invalid"));

    let e = ConfigError::EnvVarNotFound("VAR".to_string());
    assert!(e.to_string().contains("VAR"));
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
