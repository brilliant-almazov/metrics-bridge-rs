//! HTTP server with axum.
//!
//! Endpoints:
//! - GET /metrics - Prometheus metrics (with auth)
//! - GET /health - Health check (always returns 200)
//! - GET /ready - Readiness check (200 if all sources healthy)

use crate::config::{AuthType, Config};
use crate::metrics::render_metrics;
use crate::self_metrics;
use crate::source::SourceRegistry;
use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::info;

#[cfg(test)]
#[path = "server_test.rs"]
mod tests;

/// Application state shared between handlers.
pub struct AppState {
    pub config: Config,
    pub registry: SourceRegistry,
    pub cached_metrics: RwLock<Option<CachedMetrics>>,
}

#[derive(Clone)]
pub struct CachedMetrics {
    pub data: String,
    pub timestamp: Instant,
}

impl AppState {
    pub fn new(config: Config, registry: SourceRegistry) -> Self {
        Self {
            config,
            registry,
            cached_metrics: RwLock::new(None),
        }
    }
}

/// Starts the HTTP server.
pub async fn start_server(state: Arc<AppState>) {
    let config = &state.config;

    // Build router
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .with_state(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let listener = TcpListener::bind(addr).await.unwrap();

    info!("Server listening on http://{}", addr);
    info!(
        "Auth: {}, Allowed IPs: {}",
        config.server.auth.auth_type,
        if config.allowed_networks.is_empty() {
            "all".to_string()
        } else {
            format!("{:?}", config.server.allowed_ips)
        }
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

/// Authentication middleware.
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let config = &state.config;

    // Check IP whitelist
    if !config.is_ip_allowed(addr.ip()) {
        info!(ip = %addr.ip(), "Request denied: IP not allowed");
        return Err(StatusCode::FORBIDDEN);
    }

    // Check authentication
    match &config.server.auth.auth_type {
        AuthType::None => {}
        AuthType::Basic => {
            if !check_basic_auth(&request, config) {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        AuthType::Bearer => {
            if !check_bearer_auth(&request, config) {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    Ok(next.run(request).await)
}

fn check_basic_auth(request: &Request<Body>, config: &Config) -> bool {
    let auth_header = match request.headers().get(header::AUTHORIZATION) {
        Some(h) => h,
        None => return false,
    };

    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    if !auth_str.starts_with("Basic ") {
        return false;
    }

    let encoded = &auth_str[6..];
    let decoded = match STANDARD.decode(encoded) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let credentials = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let parts: Vec<&str> = credentials.splitn(2, ':').collect();
    if parts.len() != 2 {
        return false;
    }

    let (username, password) = (parts[0], parts[1]);
    let expected_user = config.server.auth.username.as_deref().unwrap_or("");
    let expected_pass = config.server.auth.password.as_deref().unwrap_or("");

    username == expected_user && password == expected_pass
}

fn check_bearer_auth(request: &Request<Body>, config: &Config) -> bool {
    let auth_header = match request.headers().get(header::AUTHORIZATION) {
        Some(h) => h,
        None => return false,
    };

    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    if !auth_str.starts_with("Bearer ") {
        return false;
    }

    let token = &auth_str[7..];
    let expected_token = config.server.auth.token.as_deref().unwrap_or("");

    token == expected_token
}

/// GET /metrics - Returns Prometheus metrics.
async fn metrics_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Check cache if enabled
    if let Some(ttl_seconds) = state.config.server.cache_ttl_seconds {
        let cache = state.cached_metrics.read().await;
        if let Some(cached) = cache.as_ref() {
            if cached.timestamp.elapsed().as_secs() < ttl_seconds {
                self_metrics::record_request(true);
                return (
                    StatusCode::OK,
                    [(
                        header::CONTENT_TYPE,
                        "text/plain; version=0.0.4; charset=utf-8",
                    )],
                    cached.data.clone(),
                );
            }
        }
        drop(cache);
    }

    let start = Instant::now();

    // Collect from all sources
    let (families, errors) = state.registry.collect_all().await;

    // Record self-metrics
    let duration = start.elapsed();
    for family in &families {
        self_metrics::record_scrape_success(&family.source, duration.as_secs_f64());
    }
    for (source, _) in &errors {
        self_metrics::record_scrape_error(source);
    }

    let success = errors.is_empty();
    self_metrics::record_request(success);

    // Render collected metrics
    let mut output = render_metrics(&families);

    // Append self-metrics from prometheus crate
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    output.push_str(&String::from_utf8_lossy(&buffer));

    // Update cache if enabled
    if state.config.server.cache_ttl_seconds.is_some() {
        let mut cache = state.cached_metrics.write().await;
        *cache = Some(CachedMetrics {
            data: output.clone(),
            timestamp: Instant::now(),
        });
    }

    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        output,
    )
}

/// GET /health - Always returns 200 OK.
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// GET /ready - Returns 200 if all sources are healthy.
async fn ready_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let results = state.registry.health_check_all().await;
    let all_healthy = results.iter().all(|(_, healthy)| *healthy);

    let status_lines: Vec<String> = results
        .iter()
        .map(|(name, healthy)| format!("{}: {}", name, if *healthy { "ok" } else { "error" }))
        .collect();

    let body = status_lines.join("\n");

    if all_healthy {
        (StatusCode::OK, body)
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, body)
    }
}
