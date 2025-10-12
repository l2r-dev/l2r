// Run with: cargo test --lib metrics_test

mod tests {
    use crate::metrics::{CoreCounterName, CoreGaugeName, Metrics};
    use bevy::prelude::Result;
    use strum::Display;

    #[derive(Display)]
    #[strum(serialize_all = "snake_case")]
    enum TestCounter {
        NewConnections,
        PacketsReceived,
    }

    #[derive(Display)]
    #[strum(serialize_all = "snake_case")]
    enum TestGauge {
        ActivePlayers,
        ActiveSessions,
        NetworkThroughputBps,
    }
    #[test]
    fn test_metrics_creation() -> Result<()> {
        let mut metrics = Metrics::new("test_".to_string());

        // Register metrics
        metrics.register_counter(TestCounter::NewConnections, "New connections")?;
        metrics.register_counter(TestCounter::PacketsReceived, "Packets received")?;
        metrics.register_gauge(TestGauge::ActivePlayers, "Active players")?;
        metrics.register_gauge(TestGauge::ActiveSessions, "Active sessions")?;
        metrics.register_gauge(TestGauge::NetworkThroughputBps, "Network throughput")?;

        let counter = metrics.counter(TestCounter::NewConnections)?;
        counter.inc();
        counter.inc_by(10);
        assert_eq!(counter.get(), 11);

        let counter = metrics.counter(TestCounter::PacketsReceived)?;
        counter.inc();
        counter.inc_by(5);
        assert_eq!(counter.get(), 6);

        metrics.gauge(TestGauge::ActivePlayers)?.set(42);
        metrics.gauge(TestGauge::ActiveSessions)?.inc();
        metrics.gauge(TestGauge::ActiveSessions)?.dec();
        metrics.gauge(TestGauge::NetworkThroughputBps)?.set(1000);
        metrics.gauge(TestGauge::NetworkThroughputBps)?.sub(500);

        let output = metrics.gather();
        assert!(!output.is_empty());
        assert!(output.contains("test_new_connections"));
        assert!(output.contains("test_active_players"));

        Ok(())
    }

    #[test]
    fn test_counter_names() {
        // Test that all counter names have proper string representation
        use strum::IntoEnumIterator;

        for counter in CoreCounterName::iter() {
            let name = counter.to_string();
            assert!(!name.is_empty());
            assert!(!name.contains(' ')); // No spaces in metric names
            assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        }
    }

    #[test]
    fn test_gauge_names() {
        // Test that all gauge names have proper string representation
        use strum::IntoEnumIterator;

        for gauge in CoreGaugeName::iter() {
            let name = gauge.to_string();
            assert!(!name.is_empty());
            assert!(!name.contains(' ')); // No spaces in metric names
            assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        }
    }

    #[test]
    fn test_metrics_prometheus_format() -> Result<()> {
        let mut metrics = Metrics::new("l2r_test_".to_string());

        metrics.register_counter(TestCounter::NewConnections, "New connections")?;
        metrics.register_gauge(TestGauge::ActivePlayers, "Active players")?;

        metrics.counter(TestCounter::NewConnections)?.inc_by(5);
        metrics.gauge(TestGauge::ActivePlayers)?.set(123);

        let output = metrics.gather();

        // Check that output is in Prometheus format
        assert!(output.contains("# HELP"));
        assert!(output.contains("# TYPE"));
        assert!(output.contains("l2r_test_"));

        // Should contain counter and gauge metrics
        assert!(output.contains("counter"));
        assert!(output.contains("gauge"));
        Ok(())
    }
}
