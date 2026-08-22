//! Prometheus metrics for telemetry-analytics.

use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec, CounterVec, GaugeVec,
    HistogramVec,
};
use std::sync::OnceLock;

pub struct Metrics {
    /// Total alerts fired, labelled by class and severity.
    pub alerts_by_class_severity_total: CounterVec,
    /// Total detector evaluation invocations.
    pub detector_evaluations_total: CounterVec,
    /// Detector evaluation latency histogram (milliseconds).
    pub detector_latency_ms: HistogramVec,
    /// Backtest precision per detector.
    pub backtest_precision: GaugeVec,
    /// Backtest recall per detector.
    pub backtest_recall: GaugeVec,
}

static METRICS: OnceLock<Metrics> = OnceLock::new();

/// Register all metrics once; subsequent calls return the cached instance.
pub fn global() -> &'static Metrics {
    METRICS.get_or_init(|| {
        let alerts_by_class_severity_total = register_counter_vec!(
            "eel_telemetry_alerts_by_class_severity_total",
            "Total telemetry alerts fired, labelled by anomaly class and severity",
            &["class", "severity"]
        )
        .expect("failed to register alerts counter");

        let detector_evaluations_total = register_counter_vec!(
            "eel_telemetry_detector_evaluations_total",
            "Total detector evaluation invocations",
            &["detector"]
        )
        .expect("failed to register evaluations counter");

        let detector_latency_ms = register_histogram_vec!(
            "eel_telemetry_detector_latency_ms_bucket",
            "Detector evaluation latency in milliseconds",
            &["detector"],
            vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0]
        )
        .expect("failed to register latency histogram");

        let backtest_precision = register_gauge_vec!(
            "eel_telemetry_backtest_precision_gauge",
            "Backtest precision score per detector",
            &["detector"]
        )
        .expect("failed to register precision gauge");

        let backtest_recall = register_gauge_vec!(
            "eel_telemetry_backtest_recall_gauge",
            "Backtest recall score per detector",
            &["detector"]
        )
        .expect("failed to register recall gauge");

        Metrics {
            alerts_by_class_severity_total,
            detector_evaluations_total,
            detector_latency_ms,
            backtest_precision,
            backtest_recall,
        }
    })
}
