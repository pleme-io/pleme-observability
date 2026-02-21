//! Prometheus metrics collection

use prometheus::{Registry, Counter, Histogram, Opts};
use crate::Result;

/// Metrics collector
pub struct MetricsCollector {
    registry: Registry,
    request_counter: Counter,
    request_duration: Histogram,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        let request_counter = Counter::with_opts(Opts::new(
            "http_requests_total",
            "Total number of HTTP requests",
        ))
        .map_err(|e| crate::ObservabilityError::MetricsError(e.to_string()))?;

        let request_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds",
            )
        )
        .map_err(|e| crate::ObservabilityError::MetricsError(e.to_string()))?;

        registry.register(Box::new(request_counter.clone()))
            .map_err(|e| crate::ObservabilityError::MetricsError(e.to_string()))?;
        registry.register(Box::new(request_duration.clone()))
            .map_err(|e| crate::ObservabilityError::MetricsError(e.to_string()))?;

        Ok(Self {
            registry,
            request_counter,
            request_duration,
        })
    }

    /// Increment request counter
    pub fn inc_requests(&self) {
        self.request_counter.inc();
    }

    /// Observe request duration
    pub fn observe_duration(&self, duration: f64) {
        self.request_duration.observe(duration);
    }

    /// Get metrics in Prometheus format
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new().unwrap();
        collector.inc_requests();
        collector.observe_duration(0.5);
        let metrics = collector.gather();
        assert!(!metrics.is_empty());
    }
}
