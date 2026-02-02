//! Tests for self-monitoring metrics.

use super::*;

#[test]
fn test_init_registers_metrics() {
    // Initialize - note that init may be called multiple times across tests
    // with different values, so we can't assert exact values for SOURCES_TOTAL
    init(2);

    // Verify UP is always set to 1
    assert_eq!(UP.get(), 1.0);

    // Verify SOURCES_TOTAL is set (positive value)
    assert!(SOURCES_TOTAL.get() > 0.0);
}

#[test]
fn test_update_process_metrics() {
    // Force START_TIME initialization
    Lazy::force(&START_TIME);

    // Update metrics
    update_process_metrics();

    // Uptime should be non-negative
    let uptime = UPTIME_SECONDS.get();
    assert!(uptime >= 0.0);
}

#[test]
fn test_record_scrape_success() {
    let source = "test-source-success";

    record_scrape_success(source, 0.5);

    // SOURCE_UP should be 1.0
    assert_eq!(SOURCE_UP.with_label_values(&[source]).get(), 1.0);
}

#[test]
fn test_record_scrape_error() {
    let source = "test-source-error";

    record_scrape_error(source);

    // SOURCE_UP should be 0.0
    assert_eq!(SOURCE_UP.with_label_values(&[source]).get(), 0.0);
}

#[test]
fn test_record_request_success() {
    // Get initial value
    let before = SCRAPE_REQUESTS_TOTAL.with_label_values(&["success"]).get();

    record_request(true);

    // Should increment by 1
    let after = SCRAPE_REQUESTS_TOTAL.with_label_values(&["success"]).get();
    assert_eq!(after, before + 1.0);
}

#[test]
fn test_record_request_error() {
    // Get initial value
    let before = SCRAPE_REQUESTS_TOTAL.with_label_values(&["error"]).get();

    record_request(false);

    // Should increment by 1
    let after = SCRAPE_REQUESTS_TOTAL.with_label_values(&["error"]).get();
    assert_eq!(after, before + 1.0);
}

#[test]
fn test_scrape_errors_increments() {
    let source = "test-errors-source";

    let before = SCRAPE_ERRORS_TOTAL.with_label_values(&[source]).get();

    record_scrape_error(source);
    record_scrape_error(source);

    let after = SCRAPE_ERRORS_TOTAL.with_label_values(&[source]).get();
    assert_eq!(after, before + 2.0);
}

#[test]
fn test_build_info_set() {
    init(1);

    // BUILD_INFO should be set to 1.0
    let version = env!("CARGO_PKG_VERSION");
    let rust_version = env!("CARGO_PKG_RUST_VERSION");
    let value = BUILD_INFO.with_label_values(&[version, rust_version]).get();
    assert_eq!(value, 1.0);
}

#[test]
fn test_source_up_toggle() {
    let source = "toggle-source";

    // Start healthy
    record_scrape_success(source, 0.1);
    assert_eq!(SOURCE_UP.with_label_values(&[source]).get(), 1.0);

    // Mark as error
    record_scrape_error(source);
    assert_eq!(SOURCE_UP.with_label_values(&[source]).get(), 0.0);

    // Recover
    record_scrape_success(source, 0.1);
    assert_eq!(SOURCE_UP.with_label_values(&[source]).get(), 1.0);
}
