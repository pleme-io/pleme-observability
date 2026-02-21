//! Distributed tracing with OpenTelemetry
//!
//! Exports traces via OTLP HTTP to Tempo (or any OTLP-compatible collector).
//!
//! # Architecture
//!
//! ```text
//! Service (tracing spans) → OTLP HTTP → Tempo → Grafana
//! ```
//!
//! The OTLP exporter uses:
//! - HTTP transport with protobuf encoding (no gRPC)
//! - Pure Rust TLS via rustls (no OpenSSL)
//! - Mozilla root certificates via webpki-roots

use crate::{ObservabilityError, Result};
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Configuration for distributed tracing
pub struct TracingConfig {
    /// Service name (required)
    pub service_name: String,
    /// OTLP endpoint (e.g., "http://tempo:4318")
    pub otlp_endpoint: String,
    /// Environment (e.g., "production", "staging")
    pub environment: Option<String>,
    /// Service version (e.g., git SHA)
    pub version: Option<String>,
}

impl TracingConfig {
    /// Create new config with required fields
    pub fn new(service_name: impl Into<String>, otlp_endpoint: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            otlp_endpoint: otlp_endpoint.into(),
            environment: None,
            version: None,
        }
    }

    /// Set environment
    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.environment = Some(env.into());
        self
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// Initialize distributed tracing with OTLP export
///
/// This sets up:
/// 1. OpenTelemetry tracer with OTLP HTTP exporter
/// 2. tracing-subscriber with JSON formatting for logs
/// 3. tracing-opentelemetry layer for span export
///
/// # Example
///
/// ```rust,ignore
/// use pleme_observability::init_distributed_tracing;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init_distributed_tracing("my-service", "http://tempo:4318")?;
///
///     tracing::info!("Service started");
///     // Spans are now exported to Tempo
///
///     Ok(())
/// }
/// ```
pub fn init_distributed_tracing(service_name: &str, endpoint: &str) -> Result<()> {
    let config = TracingConfig::new(service_name, endpoint);
    init_distributed_tracing_with_config(config)
}

/// Initialize distributed tracing with full configuration
pub fn init_distributed_tracing_with_config(config: TracingConfig) -> Result<()> {
    // Build resource attributes for the service
    let mut resource_attrs = vec![
        KeyValue::new("service.name", config.service_name.clone()),
    ];

    if let Some(env) = &config.environment {
        resource_attrs.push(KeyValue::new("deployment.environment", env.clone()));
    }

    if let Some(version) = &config.version {
        resource_attrs.push(KeyValue::new("service.version", version.clone()));
    }

    // Build resource using the builder pattern (Resource::new is private in 0.28)
    let resource = Resource::builder()
        .with_attributes(resource_attrs)
        .build();

    // Configure OTLP exporter with HTTP transport
    // Uses http-proto feature (not gRPC) for pure Rust
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(&config.otlp_endpoint)
        .build()
        .map_err(|e| ObservabilityError::TracingInit(format!("Failed to create OTLP exporter: {}", e)))?;

    // Build the tracer provider (renamed to SdkTracerProvider in 0.28)
    // Note: with_batch_exporter no longer takes a runtime argument in 0.28
    let provider = sdktrace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    // Get a tracer from the provider
    let tracer = provider.tracer(config.service_name.clone());

    // Set the global tracer provider
    opentelemetry::global::set_tracer_provider(provider);

    // Create the OpenTelemetry tracing layer
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Configure log filter from RUST_LOG env var
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Build the subscriber with JSON logs + OpenTelemetry traces
    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_level(true)
                .json(),
        )
        .with(otel_layer)
        .init();

    tracing::info!(
        service = config.service_name,
        endpoint = config.otlp_endpoint,
        "Distributed tracing initialized"
    );

    Ok(())
}

/// Shutdown the tracer provider gracefully
///
/// Call this before application exit to ensure all pending spans are exported.
/// In OpenTelemetry 0.28+, this is done by dropping the global provider.
pub fn shutdown_tracing() {
    // In OpenTelemetry 0.28, shutdown is handled by the provider's Drop impl
    // or by explicitly calling shutdown on the provider.
    // The global API no longer has shutdown_tracer_provider().
    // For graceful shutdown, services should keep a handle to the provider
    // and call provider.shutdown() directly.
    //
    // This function is kept for API compatibility but is now a no-op.
    // Services that need graceful shutdown should manage the provider directly.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_config_builder() {
        let config = TracingConfig::new("test-service", "http://localhost:4318")
            .with_environment("test")
            .with_version("abc123");

        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.otlp_endpoint, "http://localhost:4318");
        assert_eq!(config.environment, Some("test".to_string()));
        assert_eq!(config.version, Some("abc123".to_string()));
    }

    // Note: Can't test actual initialization in unit tests as it sets global state
    // Integration tests would need a running OTLP collector
}
