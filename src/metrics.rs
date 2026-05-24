use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder,
};

pub struct ProxyPrometheusMetrics {
    registry: Registry,
    requests_total: IntCounterVec,
    request_duration: HistogramVec,
    errors_total: IntCounterVec,
}

impl Default for ProxyPrometheusMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyPrometheusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        let requests_total = IntCounterVec::new(
            Opts::new("nexa_proxy_requests_total", "Total proxy requests"),
            &["domain", "status"],
        )
        .unwrap();

        let request_duration = HistogramVec::new(
            HistogramOpts::new(
                "nexa_proxy_request_duration_seconds",
                "Proxy request duration in seconds",
            ),
            &["domain"],
        )
        .unwrap();

        let errors_total = IntCounterVec::new(
            Opts::new("nexa_proxy_errors_total", "Total proxy errors"),
            &["domain", "error_type"],
        )
        .unwrap();

        registry.register(Box::new(requests_total.clone())).unwrap();
        registry
            .register(Box::new(request_duration.clone()))
            .unwrap();
        registry.register(Box::new(errors_total.clone())).unwrap();

        Self {
            registry,
            requests_total,
            request_duration,
            errors_total,
        }
    }

    pub fn record_request(&self, domain: &str, status: u16, duration_secs: f64) {
        self.requests_total
            .with_label_values(&[domain, &status.to_string()])
            .inc();
        self.request_duration
            .with_label_values(&[domain])
            .observe(duration_secs);
    }

    pub fn record_error(&self, domain: &str, error_type: &str) {
        self.errors_total
            .with_label_values(&[domain, error_type])
            .inc();
    }

    pub fn encode(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_request_appears_in_output() {
        let m = ProxyPrometheusMetrics::new();
        m.record_request("api.example.com", 200, 0.05);
        let output = m.encode();
        assert!(output.contains("nexa_proxy_requests_total"));
        assert!(output.contains("nexa_proxy_request_duration_seconds"));
        assert!(output.contains("api.example.com"));
    }

    #[test]
    fn record_error_appears_in_output() {
        let m = ProxyPrometheusMetrics::new();
        m.record_error("api.example.com", "connection_refused");
        let output = m.encode();
        assert!(output.contains("nexa_proxy_errors_total"));
        assert!(output.contains("connection_refused"));
    }

    #[test]
    fn multiple_domains_tracked_independently() {
        let m = ProxyPrometheusMetrics::new();
        m.record_request("api.example.com", 200, 0.05);
        m.record_request("web.example.com", 502, 0.1);
        let output = m.encode();
        assert!(output.contains("api.example.com"));
        assert!(output.contains("web.example.com"));
    }
}
