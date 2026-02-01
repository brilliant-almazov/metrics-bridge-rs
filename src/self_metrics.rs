//! Self-monitoring metrics for the exporter itself.

use once_cell::sync::Lazy;
use prometheus::{
    register_counter_vec, register_gauge, register_gauge_vec, register_histogram_vec,
    CounterVec, Gauge, GaugeVec, HistogramVec,
};

/// Total number of scrape requests.
pub static SCRAPE_REQUESTS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "metrics_bridge_scrape_requests_total",
        "Total number of scrape requests",
        &["status"]
    )
    .expect("Failed to register scrape_requests_total")
});

/// Duration of metric collection from sources.
pub static SCRAPE_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "metrics_bridge_scrape_duration_seconds",
        "Duration of metric collection in seconds",
        &["source"],
        vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]
    )
    .expect("Failed to register scrape_duration_seconds")
});

/// Total number of scrape errors per source.
pub static SCRAPE_ERRORS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "metrics_bridge_scrape_errors_total",
        "Total number of scrape errors",
        &["source"]
    )
    .expect("Failed to register scrape_errors_total")
});

/// Number of configured sources.
pub static SOURCES_TOTAL: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "metrics_bridge_sources_total",
        "Total number of configured sources"
    )
    .expect("Failed to register sources_total")
});

/// Health status of each source (1 = healthy, 0 = unhealthy).
pub static SOURCE_UP: Lazy<GaugeVec> = Lazy::new(|| {
    register_gauge_vec!(
        "metrics_bridge_source_up",
        "Health status of source (1 = healthy, 0 = unhealthy)",
        &["source"]
    )
    .expect("Failed to register source_up")
});

/// Exporter up status (always 1 when running).
pub static UP: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!("metrics_bridge_up", "Exporter is running (always 1)").expect("Failed to register up")
});

/// Build information.
pub static BUILD_INFO: Lazy<GaugeVec> = Lazy::new(|| {
    register_gauge_vec!(
        "metrics_bridge_build_info",
        "Build information",
        &["version", "rust_version"]
    )
    .expect("Failed to register build_info")
});

/// Initialize all metrics.
pub fn init(sources_count: usize) {
    // Touch lazy statics to register them
    Lazy::force(&SCRAPE_REQUESTS_TOTAL);
    Lazy::force(&SCRAPE_DURATION_SECONDS);
    Lazy::force(&SCRAPE_ERRORS_TOTAL);
    Lazy::force(&SOURCES_TOTAL);
    Lazy::force(&SOURCE_UP);
    Lazy::force(&UP);
    Lazy::force(&BUILD_INFO);

    // Set static values
    SOURCES_TOTAL.set(sources_count as f64);
    UP.set(1.0);
    BUILD_INFO
        .with_label_values(&[env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_RUST_VERSION")])
        .set(1.0);
}

/// Record a successful scrape.
pub fn record_scrape_success(source: &str, duration_secs: f64) {
    SCRAPE_DURATION_SECONDS
        .with_label_values(&[source])
        .observe(duration_secs);
    SOURCE_UP.with_label_values(&[source]).set(1.0);
}

/// Record a failed scrape.
pub fn record_scrape_error(source: &str) {
    SCRAPE_ERRORS_TOTAL.with_label_values(&[source]).inc();
    SOURCE_UP.with_label_values(&[source]).set(0.0);
}

/// Record scrape request.
pub fn record_request(success: bool) {
    let status = if success { "success" } else { "error" };
    SCRAPE_REQUESTS_TOTAL.with_label_values(&[status]).inc();
}
