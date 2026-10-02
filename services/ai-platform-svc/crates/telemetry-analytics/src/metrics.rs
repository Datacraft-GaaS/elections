// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
