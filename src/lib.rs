//! # pleme-observability
//!
//! Observability library for Pleme platform services.
//!
//! ## Features
//!
//! - **Structured Logging** - JSON logging with tracing
//! - **Distributed Tracing** - OpenTelemetry integration (feature-gated)
//! - **Metrics** - Prometheus metric definition macros and tracking helpers
//! - **Context Propagation** - W3C Trace Context
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use pleme_observability::init_observability;
//!
//! fn main() {
//!     // Auto-detects: uses OTel if OTEL_EXPORTER_OTLP_ENDPOINT is set,
//!     // otherwise falls back to basic JSON tracing.
//!     init_observability("my-service");
//!     tracing::info!("Service started");
//! }
//! ```

pub mod logging;

#[cfg(feature = "distributed-tracing")]
pub mod tracing;

pub mod metrics;

#[macro_use]
pub mod macros;

pub mod tracking;

pub use logging::init_tracing;

#[cfg(feature = "distributed-tracing")]
pub use tracing::{
    init_distributed_tracing, init_distributed_tracing_with_config, shutdown_tracing, TracingConfig,
};

pub use metrics::MetricsCollector;

use thiserror::Error;

/// Observability errors
#[derive(Error, Debug)]
pub enum ObservabilityError {
    #[error("Tracing initialization failed: {0}")]
    TracingInit(String),

    #[error("Metrics collection failed: {0}")]
    MetricsError(String),
}

/// Result type for observability operations
pub type Result<T> = std::result::Result<T, ObservabilityError>;

/// Initialize observability with automatic environment detection.
///
/// - If `OTEL_EXPORTER_OTLP_ENDPOINT` is set: initializes distributed tracing with OTel
/// - Otherwise: initializes basic JSON tracing (local dev)
///
/// This lets the same code work in dev (no OTel) and prod (with OTel) without code changes.
pub fn init_observability(service_name: &str) {
    match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        #[cfg(feature = "distributed-tracing")]
        Ok(endpoint) => {
            let mut config = TracingConfig::new(service_name, &endpoint);

            if let Ok(env) = std::env::var("DEPLOYMENT_ENV") {
                config = config.with_environment(env);
            }
            if let Ok(version) = std::env::var("SERVICE_VERSION") {
                config = config.with_version(version);
            }

            if let Err(e) = init_distributed_tracing_with_config(config) {
                eprintln!(
                    "WARNING: Failed to initialize distributed tracing: {}. Falling back to basic tracing.",
                    e
                );
                init_tracing(service_name);
            }
        }
        #[cfg(not(feature = "distributed-tracing"))]
        Ok(_endpoint) => {
            eprintln!(
                "WARNING: OTEL_EXPORTER_OTLP_ENDPOINT is set but distributed-tracing feature is not enabled. Using basic tracing."
            );
            init_tracing(service_name);
        }
        Err(_) => {
            init_tracing(service_name);
        }
    }
}
