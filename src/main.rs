//! metrics-bridge - Fast Prometheus metrics exporter for promphp Redis storage.
//!
//! Endpoints:
//! - GET /metrics - Prometheus metrics (with auth)
//! - GET /health - Health check
//! - GET /ready - Readiness check

// Memory allocator configuration (Linux only)
#[cfg(all(target_os = "linux", not(target_env = "msvc")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use metrics_bridge::{self_metrics, server, Config, SourceRegistry};
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("metrics_bridge=info".parse().unwrap()),
        )
        .init();

    info!("metrics-bridge v{} starting...", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    info!("Configured {} source(s)", config.sources.len());
    for source in &config.sources {
        info!(
            "  - {} ({}) prefix={}",
            source.name, source.source_type, source.prefix
        );
    }

    // Initialize source registry
    let registry = match SourceRegistry::from_config(&config) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to initialize sources: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize self-metrics
    self_metrics::init(registry.len());

    // Create application state
    let state = Arc::new(server::AppState::new(config, registry));

    // Start server with graceful shutdown
    let server_state = state.clone();
    let server_handle = tokio::spawn(async move {
        server::start_server(server_state).await;
    });

    // Wait for shutdown signal
    shutdown_signal().await;

    info!("Shutting down...");

    // Abort server (graceful shutdown not fully implemented in axum::serve)
    server_handle.abort();

    info!("Goodbye!");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C");
        }
        _ = terminate => {
            info!("Received SIGTERM");
        }
    }
}
