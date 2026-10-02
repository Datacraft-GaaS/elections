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

//! DropAtScaleDetector: cross-references DropSignal (accel > 3g) with the device's
//! assigned custody state from eel_cover_state_transitions (M20).
//!
//! Reference: ISO 16750-3 (mechanical load requirements for electronic equipment in vehicles)
//! adapted to kit custody semantics.
//!
//! Severity mapping:
//!   station-open + voting-active during drop  -> Critical
//!   in-transit during drop                    -> Warn
//!   all other custody states                  -> Info

use crate::detector::{Alert, AlertSeverity, Detector, DetectorError, SignalStream};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Configuration for DropAtScaleDetector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropDetectorConfig {
    /// Rule id registered in the RuleRegistry.
    pub rule_id: String,
    /// Acceleration threshold in g-force above which a signal is classified as a drop.
    pub accel_threshold_g: f64,
    pub enabled: bool,
}

impl Default for DropDetectorConfig {
    fn default() -> Self {
        Self {
            rule_id: "drop_at_scale".into(),
            accel_threshold_g: 3.0,
            enabled: true,
        }
    }
}

/// Custody states derived from eel_cover_state_transitions (M20).
#[derive(Debug, Clone, PartialEq, Eq)]
enum CustodyState {
    StationOpenVotingActive,
    InTransit,
    Other,
}

fn parse_custody_state(s: &str) -> CustodyState {
    match s {
        "station_open_voting_active" => CustodyState::StationOpenVotingActive,
        "in_transit" => CustodyState::InTransit,
        _ => CustodyState::Other,
    }
}

pub struct DropAtScaleDetector {
    config: DropDetectorConfig,
}

impl DropAtScaleDetector {
    pub fn new(config: DropDetectorConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Detector for DropAtScaleDetector {
    fn name(&self) -> &str {
        "DropAtScaleDetector"
    }

    fn rule_id(&self) -> &str {
        &self.config.rule_id
    }

    fn enabled(&self) -> bool {
        self.config.enabled
    }

    async fn evaluate(&self, signals: &SignalStream) -> Result<Vec<Alert>, DetectorError> {
        let mut alerts = Vec::new();

        for signal in signals {
            if signal.signal_type != "drop" {
                continue;
            }
            let accel_g = signal.payload["accel_g"]
                .as_f64()
                .unwrap_or(0.0);
            if accel_g <= self.config.accel_threshold_g {
                continue;
            }
            let custody = signal.payload["custody_state"]
                .as_str()
                .map(parse_custody_state)
                .unwrap_or(CustodyState::Other);

            let (severity, note) = match custody {
                CustodyState::StationOpenVotingActive => (
                    AlertSeverity::Critical,
                    "drop during station-open + voting-active",
                ),
                CustodyState::InTransit => (AlertSeverity::Warn, "drop during in-transit custody"),
                CustodyState::Other => (AlertSeverity::Info, "drop in non-critical custody state"),
            };

            let mut alert = Alert::new(
                &self.config.rule_id,
                "drop_at_scale",
                severity,
                serde_json::json!({
                    "accel_g": accel_g,
                    "custody_state": signal.payload["custody_state"],
                    "note": note,
                }),
            )
            .with_device(signal.device_id.clone());

            if let Some(station) = signal.payload["station_id"].as_str() {
                alert = alert.with_station(station);
            }
            alerts.push(alert);
        }
        Ok(alerts)
    }
}
