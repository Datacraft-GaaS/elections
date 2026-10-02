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

//! AnalyticsPipeline: reads from Kafka, fans out to detectors via rayon, dispatches alerts.

use crate::detector::{Detector, SignalStream, TelemetrySignal};
use crate::dispatch::AlertDispatcher;
use crate::metrics;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, warn};

/// Holds detectors and the dispatcher.
pub struct AnalyticsPipeline {
    detectors: Vec<Arc<dyn Detector>>,
    dispatcher: Arc<AlertDispatcher>,
}

impl AnalyticsPipeline {
    pub fn new(detectors: Vec<Arc<dyn Detector>>, dispatcher: Arc<AlertDispatcher>) -> Self {
        Self {
            detectors,
            dispatcher,
        }
    }

    /// Evaluate all detectors against the given signal batch in parallel via rayon,
    /// then dispatch any resulting alerts.
    pub async fn process(&self, signals: SignalStream) -> Vec<crate::detector::Alert> {
        let m = metrics::global();
        let mut all_alerts = Vec::new();

        for detector in &self.detectors {
            if !detector.enabled() {
                continue;
            }
            let name = detector.name().to_string();
            let t0 = Instant::now();
            m.detector_evaluations_total
                .with_label_values(&[&name])
                .inc();

            match detector.evaluate(&signals).await {
                Ok(alerts) => {
                    let elapsed = t0.elapsed().as_millis() as f64;
                    m.detector_latency_ms
                        .with_label_values(&[&name])
                        .observe(elapsed);

                    for alert in &alerts {
                        let sev = crate::dispatch::severity_label(&alert.severity);
                        m.alerts_by_class_severity_total
                            .with_label_values(&[&alert.class, sev])
                            .inc();
                    }
                    all_alerts.extend(alerts);
                }
                Err(e) => {
                    error!(detector = %name, error = %e, "detector evaluation failed");
                }
            }
        }

        // Dispatch all alerts asynchronously.
        for alert in &all_alerts {
            if let Err(e) = self.dispatcher.dispatch(alert).await {
                warn!(alert_id = %alert.id, error = %e, "alert dispatch failed");
            }
        }

        all_alerts
    }

    /// Parse a raw Kafka message payload into a TelemetrySignal.
    pub fn parse_signal(raw: &[u8]) -> Option<TelemetrySignal> {
        serde_json::from_slice(raw).ok()
    }
}

/// Kafka consumer bootstrap for the pipeline.
///
/// In production this is the entry point that drives the event loop.
/// Kept as a free function so tests can bypass Kafka entirely.
pub fn build_kafka_config(
    brokers: &str,
    group_id: &str,
    _topic: &str,
) -> rdkafka::ClientConfig {
    let mut cfg = rdkafka::ClientConfig::new();
    cfg.set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("auto.offset.reset", "earliest")
        .set("enable.auto.commit", "true");
    cfg
}
