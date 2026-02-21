//! Structured logging setup

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing subscriber
pub fn init_tracing(service_name: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_level(true)
                .json(),
        )
        .init();

    tracing::info!(service = service_name, "Tracing initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracing() {
        // Can't actually test init_tracing as it can only be called once per process
        // But we can verify it compiles and has correct signature
        let _ = init_tracing;
    }
}
