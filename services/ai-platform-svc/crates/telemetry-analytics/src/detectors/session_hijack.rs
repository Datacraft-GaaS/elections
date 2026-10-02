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

//! SessionHijackDetector: extends HARD-14 behavioural-biometrics Mahalanobis score
//! with device location, battery state, and attestation status.
//!
//! Reference: Mahalanobis distance for outlier detection (Mahalanobis 1936);
//! applied here as a combined feature vector over multiple device signals.
//!
//! Severity: Critical when combined score exceeds threshold.

use crate::detector::{Alert, AlertSeverity, Detector, DetectorError, SignalStream};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for SessionHijackDetector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHijackConfig {
    pub rule_id: String,
    /// Combined score threshold above which a session hijack is suspected.
    /// Dimensionless Mahalanobis-derived composite (see score() method).
    pub score_threshold: f64,
    pub enabled: bool,
}

impl Default for SessionHijackConfig {
    fn default() -> Self {
        Self {
            rule_id: "session_hijack".into(),
            score_threshold: 5.0,
            enabled: true,
        }
    }
}

/// Feature vector for a session at a point in time.
#[derive(Debug, Clone)]
struct SessionFeatures {
    mahalanobis_distance: f64,
    location_changed: bool,
    battery_critically_low: bool,
    attestation_failed: bool,
}

impl SessionFeatures {
    /// Composite score: weighted sum of features.
    ///
    /// Weights:
    ///   mahalanobis_distance * 1.0
    ///   location_changed     * 2.0
    ///   battery_critically_low * 0.5
    ///   attestation_failed   * 3.0
    fn score(&self) -> f64 {
        self.mahalanobis_distance
            + if self.location_changed { 2.0 } else { 0.0 }
            + if self.battery_critically_low { 0.5 } else { 0.0 }
            + if self.attestation_failed { 3.0 } else { 0.0 }
    }
}

pub struct SessionHijackDetector {
    config: SessionHijackConfig,
}

impl SessionHijackDetector {
    pub fn new(config: SessionHijackConfig) -> Self {
        Self { config }
    }

    fn extract_features(signals: &[&crate::detector::TelemetrySignal]) -> Option<SessionFeatures> {
        let biometric = signals
            .iter()
            .find(|s| s.signal_type == "biometric_score")?;
        let md = biometric.payload["mahalanobis_distance"].as_f64()?;

        let location_changed = signals
            .iter()
            .any(|s| s.signal_type == "location" && s.payload["changed"].as_bool().unwrap_or(false));
        let battery_critically_low = signals.iter().any(|s| {
            s.signal_type == "battery"
                && s.payload["level_pct"].as_f64().unwrap_or(100.0) < 10.0
        });
        let attestation_failed = signals.iter().any(|s| {
            s.signal_type == "attestation"
                && !s.payload["passed"].as_bool().unwrap_or(true)
        });

        Some(SessionFeatures {
            mahalanobis_distance: md,
            location_changed,
            battery_critically_low,
            attestation_failed,
        })
    }
}

#[async_trait]
impl Detector for SessionHijackDetector {
    fn name(&self) -> &str {
        "SessionHijackDetector"
    }

    fn rule_id(&self) -> &str {
        &self.config.rule_id
    }

    fn enabled(&self) -> bool {
        self.config.enabled
    }

    async fn evaluate(&self, signals: &SignalStream) -> Result<Vec<Alert>, DetectorError> {
        let mut alerts = Vec::new();

        // Group signals by session_id.
        let mut by_session: HashMap<&str, Vec<&crate::detector::TelemetrySignal>> = HashMap::new();
        for signal in signals {
            if let Some(session_id) = signal.payload["session_id"].as_str() {
                by_session.entry(session_id).or_default().push(signal);
            }
        }

        for (session_id, sess_signals) in &by_session {
            let Some(features) = Self::extract_features(sess_signals) else {
                continue;
            };
            let score = features.score();
            if score > self.config.score_threshold {
                let officer_id = sess_signals
                    .first()
                    .and_then(|s| s.payload["officer_id"].as_str())
                    .unwrap_or("")
                    .to_string();
                let device_id = sess_signals
                    .first()
                    .map(|s| s.device_id.as_str())
                    .unwrap_or("")
                    .to_string();

                let alert = Alert::new(
                    &self.config.rule_id,
                    "session_hijack",
                    AlertSeverity::Critical,
                    serde_json::json!({
                        "session_id": session_id,
                        "composite_score": score,
                        "threshold": self.config.score_threshold,
                        "mahalanobis_distance": features.mahalanobis_distance,
                        "location_changed": features.location_changed,
                        "battery_critically_low": features.battery_critically_low,
                        "attestation_failed": features.attestation_failed,
                    }),
                )
                .with_device(device_id)
                .with_officer(officer_id);

                alerts.push(alert);
            }
        }
        Ok(alerts)
    }
}
