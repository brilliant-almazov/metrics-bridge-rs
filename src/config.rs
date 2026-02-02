//! Configuration management for metrics-bridge.
//!
//! Configuration is loaded from YAML with environment variable substitution.
//! Supports CONFIG_BASE64 env var or config.yaml file.

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use thiserror::Error;

/// Configuration error types.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required config: {0}")]
    MissingValue(String),

    #[error("Config file error: {0}")]
    FileError(String),

    #[error("YAML parse error: {0}")]
    YamlError(String),

    #[error("Base64 decode error: {0}")]
    Base64Error(String),

    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
}

/// Authentication type for /metrics endpoint.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    #[default]
    None,
    Basic,
    Bearer,
}

impl fmt::Display for AuthType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthType::None => write!(f, "none"),
            AuthType::Basic => write!(f, "basic"),
            AuthType::Bearer => write!(f, "bearer"),
        }
    }
}

/// TLS configuration.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TlsConfig {
    pub cert: Option<String>,
    pub key: Option<String>,
}

impl TlsConfig {
    pub fn is_enabled(&self) -> bool {
        self.cert.is_some() && self.key.is_some()
    }
}

/// Authentication configuration.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default, rename = "type")]
    pub auth_type: AuthType,
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
}

/// Server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub allowed_ips: Vec<String>,
    #[serde(default)]
    pub tls: TlsConfig,
    /// Cache TTL in seconds. 0 = no caching (default).
    #[serde(default)]
    pub cache_ttl_seconds: u64,
    /// Optional GZIP compression level (1-9). If set, responses are compressed.
    #[serde(default)]
    pub gzip_level: Option<u32>,
}

fn default_port() -> u16 {
    9090
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            auth: AuthConfig::default(),
            allowed_ips: Vec::new(),
            tls: TlsConfig::default(),
            cache_ttl_seconds: 0,
            gzip_level: None,
        }
    }
}

/// Source type enum.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceType {
    PromphpRedis,
}

/// Label encoding format for promphp metrics.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LabelFormat {
    /// Auto-detect format (try JSON first, then base64).
    #[default]
    Auto,
    /// Raw JSON array format: `["value1", "value2"]`
    Json,
    /// Base64-encoded JSON array format.
    Base64,
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::PromphpRedis => write!(f, "promphp-redis"),
        }
    }
}

/// Source configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub source_type: SourceType,
    pub redis_url: String,
    #[serde(default = "default_prefix")]
    pub prefix: String,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    /// Cache TTL in seconds for this source. 0 = no caching (default).
    #[serde(default)]
    pub cache_ttl_seconds: u64,
    /// Label encoding format. Default: auto-detect.
    #[serde(default)]
    pub label_format: LabelFormat,
}

fn default_prefix() -> String {
    "PROMETHEUS_".to_string()
}

/// YAML configuration structure.
#[derive(Debug, Deserialize)]
struct YamlConfig {
    #[serde(default)]
    server: ServerConfig,
    #[serde(default)]
    sources: Vec<SourceConfig>,
}

/// Application configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub sources: Vec<SourceConfig>,
    pub allowed_networks: Vec<ipnet::IpNet>,
}

impl Config {
    /// Loads configuration from YAML sources.
    ///
    /// Priority:
    /// 1. Base64-encoded YAML in CONFIG_BASE64 env var
    /// 2. YAML file specified in CONFIG_FILE env var
    /// 3. Default config.yaml file
    pub fn load() -> Result<Self, ConfigError> {
        let yaml_content = Self::load_yaml_content()?;
        let expanded = expand_env_vars(&yaml_content)?;
        let yaml_config: YamlConfig =
            serde_yaml::from_str(&expanded).map_err(|e| ConfigError::YamlError(e.to_string()))?;

        // Validate auth config
        Self::validate_auth(&yaml_config.server.auth)?;

        // Validate sources
        if yaml_config.sources.is_empty() {
            return Err(ConfigError::MissingValue("sources".to_string()));
        }

        // Parse allowed IPs into networks
        let allowed_networks = Self::parse_allowed_ips(&yaml_config.server.allowed_ips)?;

        Ok(Self {
            server: yaml_config.server,
            sources: yaml_config.sources,
            allowed_networks,
        })
    }

    fn load_yaml_content() -> Result<String, ConfigError> {
        // Try CONFIG_BASE64 first
        if let Ok(b64) = env::var("CONFIG_BASE64") {
            let bytes = STANDARD
                .decode(&b64)
                .map_err(|e| ConfigError::Base64Error(e.to_string()))?;
            return String::from_utf8(bytes).map_err(|e| ConfigError::Base64Error(e.to_string()));
        }

        // Try CONFIG_FILE env var
        if let Ok(path) = env::var("CONFIG_FILE") {
            if Path::new(&path).exists() {
                return fs::read_to_string(&path)
                    .map_err(|e| ConfigError::FileError(e.to_string()));
            }
        }

        // Try default config.yaml
        if Path::new("config.yaml").exists() {
            return fs::read_to_string("config.yaml")
                .map_err(|e| ConfigError::FileError(e.to_string()));
        }

        Err(ConfigError::FileError(
            "No config found. Provide CONFIG_BASE64 env var or config.yaml file".to_string(),
        ))
    }

    fn validate_auth(auth: &AuthConfig) -> Result<(), ConfigError> {
        match auth.auth_type {
            AuthType::Basic => {
                if auth.username.is_none() || auth.password.is_none() {
                    return Err(ConfigError::InvalidValue(
                        "Basic auth requires username and password".to_string(),
                    ));
                }
            }
            AuthType::Bearer => {
                if auth.token.is_none() {
                    return Err(ConfigError::InvalidValue(
                        "Bearer auth requires token".to_string(),
                    ));
                }
            }
            AuthType::None => {}
        }
        Ok(())
    }

    fn parse_allowed_ips(ips: &[String]) -> Result<Vec<ipnet::IpNet>, ConfigError> {
        ips.iter()
            .map(|ip| {
                // Try parsing as network first
                if ip.contains('/') {
                    ip.parse::<ipnet::IpNet>()
                        .map_err(|e| ConfigError::InvalidValue(format!("Invalid IP network: {e}")))
                } else {
                    // Parse as single IP and convert to /32 or /128
                    let addr: IpAddr = ip.parse().map_err(|e| {
                        ConfigError::InvalidValue(format!("Invalid IP address: {e}"))
                    })?;
                    Ok(ipnet::IpNet::from(addr))
                }
            })
            .collect()
    }

    /// Check if an IP is allowed (empty list = allow all).
    pub fn is_ip_allowed(&self, ip: IpAddr) -> bool {
        if self.allowed_networks.is_empty() {
            return true;
        }
        self.allowed_networks.iter().any(|net| net.contains(&ip))
    }
}

/// Expands environment variables in the format ${VAR} in the input string.
fn expand_env_vars(content: &str) -> Result<String, ConfigError> {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut errors = Vec::new();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let var_name: String = chars.by_ref().take_while(|&c| c != '}').collect();
            match env::var(&var_name) {
                Ok(value) => result.push_str(&value),
                Err(_) => errors.push(var_name),
            }
        } else {
            result.push(c);
        }
    }

    if !errors.is_empty() {
        return Err(ConfigError::EnvVarNotFound(errors.join(", ")));
    }

    Ok(result)
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
