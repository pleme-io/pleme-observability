//! Generic infrastructure tracking helpers.
//!
//! These are free functions that accept Prometheus metric references as parameters.
//! Each service wraps them with zero-arg helpers that pass in their own global metrics.
//!
//! This avoids coupling the platform crate to any specific global or service.

use prometheus::{HistogramVec, IntCounterVec, IntGauge};
use std::time::Duration;

/// Track HTTP request metrics (counter + duration histogram).
#[inline]
pub fn track_request_metrics(
    requests_total: &IntCounterVec,
    request_duration: &HistogramVec,
    method: &str,
    path: &str,
    status: &str,
    duration: Duration,
) {
    requests_total
        .with_label_values(&[method, path, status])
        .inc();
    request_duration
        .with_label_values(&[method, path])
        .observe(duration.as_secs_f64());
}

/// Track a GraphQL operation (query or mutation counter).
#[inline]
pub fn track_graphql_operation(counter: &IntCounterVec, operation_name: &str) {
    counter.with_label_values(&[operation_name]).inc();
}

/// Track GraphQL execution with duration and optional error tracking.
#[inline]
pub fn track_graphql_execution(
    duration_histogram: &HistogramVec,
    errors_total: &IntCounterVec,
    operation_type: &str,
    operation_name: &str,
    duration_secs: f64,
    has_errors: bool,
    error_types: &[&str],
) {
    duration_histogram
        .with_label_values(&[operation_type, operation_name])
        .observe(duration_secs);

    if has_errors {
        for error_type in error_types {
            errors_total
                .with_label_values(&[operation_type, error_type])
                .inc();
        }
    }
}

/// Track a Redis operation with timing.
#[inline]
pub fn track_redis_operation(
    operations_total: &IntCounterVec,
    operation_duration: &HistogramVec,
    operation: &str,
    result: &str,
    duration_secs: f64,
) {
    operations_total
        .with_label_values(&[operation, result])
        .inc();
    operation_duration
        .with_label_values(&[operation])
        .observe(duration_secs);
}

/// Track an S3 operation duration.
#[inline]
pub fn track_s3_operation(s3_duration: &HistogramVec, operation: &str, duration_secs: f64) {
    s3_duration
        .with_label_values(&[operation])
        .observe(duration_secs);
}

/// Track a startup or shutdown phase duration.
#[inline]
pub fn track_phase_duration(histogram: &HistogramVec, phase: &str, duration_secs: f64) {
    histogram.with_label_values(&[phase]).observe(duration_secs);
}

/// Track a health check execution.
#[inline]
pub fn track_health_check(
    duration_histogram: &HistogramVec,
    results_counter: &IntCounterVec,
    check_type: &str,
    duration_secs: f64,
    healthy: bool,
) {
    duration_histogram
        .with_label_values(&[check_type])
        .observe(duration_secs);

    let result = if healthy { "healthy" } else { "unhealthy" };
    results_counter
        .with_label_values(&[check_type, result])
        .inc();
}

/// Convert HTTP status code to static string slice (zero allocation).
#[inline]
#[allow(clippy::match_overlapping_arm)]
pub fn status_to_str(status: u16) -> &'static str {
    match status {
        200 => "200",
        201 => "201",
        204 => "204",
        301 => "301",
        302 => "302",
        304 => "304",
        400 => "400",
        401 => "401",
        403 => "403",
        404 => "404",
        405 => "405",
        409 => "409",
        422 => "422",
        429 => "429",
        500 => "500",
        502 => "502",
        503 => "503",
        504 => "504",
        100..=199 => "1xx",
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "other",
    }
}

/// Normalize path to static string to avoid unbounded cardinality.
#[inline]
pub fn normalize_path(path: &str) -> &'static str {
    match path {
        "/" => "/",
        "/graphql" => "/graphql",
        "/health" => "/health",
        "/health/ready" => "/health/ready",
        "/health/live" => "/health/live",
        "/metrics" => "/metrics",
        p if p.starts_with("/health") => "/health/*",
        p if p.starts_with("/admin") => "/admin/*",
        p if p.starts_with("/api") => "/api/*",
        _ => "/other",
    }
}

/// Lifecycle state constants for service state gauges.
pub mod lifecycle_state {
    pub const STARTING: i64 = 0;
    pub const RUNNING: i64 = 1;
    pub const DRAINING: i64 = 2;
    pub const SHUTDOWN: i64 = 3;
}

/// Set the current lifecycle state on a gauge.
#[inline]
pub fn set_lifecycle_state(gauge: &IntGauge, state: i64) {
    gauge.set(state);
}

/// Generate Prometheus metrics text output from a registry.
pub async fn metrics_handler(registry: &prometheus::Registry) -> Result<String, &'static str> {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();

    prometheus::Encoder::encode(&encoder, &metric_families, &mut buffer)
        .map_err(|_| "Failed to encode metrics")?;

    String::from_utf8(buffer).map_err(|_| "Metrics output is not valid UTF-8")
}
