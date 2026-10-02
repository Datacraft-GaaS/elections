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

//! RfidAbsenceAnomalyDetector: kit expected to see a specific ballot-box RFID
//! (from custody schedule) but has not seen it for > 15 min during transit.
//!
//! Cross-references eel_cover_state_transitions to determine expected RFID pairs.
//!
//! Reference: ISO 18000-6C (RFID air interface standard for UHF passive RFID).
//! Absence for > 15 min during active-transit custody raises a Warn alert.

use crate::detector::{Alert, AlertSeverity, Detector, DetectorError, SignalStream};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Configuration for RfidAbsenceAnomalyDetector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfidAbsenceConfig {
    pub rule_id: String,
    /// Maximum allowed absence duration in minutes before firing.
    pub absence_threshold_minutes: i64,
    pub enabled: bool,
}

impl Default for RfidAbsenceConfig {
    fn default() -> Self {
        Self {
            rule_id: "rfid_absence_anomaly".into(),
            absence_threshold_minutes: 15,
            enabled: true,
        }
    }
}

pub struct RfidAbsenceAnomalyDetector {
    config: RfidAbsenceConfig,
    /// expected_rfid[device_id] = set of ballot-box RFID tags the device should see during transit.
    expected_rfid: HashMap<String, HashSet<String>>,
}

impl RfidAbsenceAnomalyDetector {
    pub fn new(config: RfidAbsenceConfig, expected_rfid: HashMap<String, HashSet<String>>) -> Self {
        Self {
            config,
            expected_rfid,
        }
    }
}

#[async_trait]
impl Detector for RfidAbsenceAnomalyDetector {
    fn name(&self) -> &str {
        "RfidAbsenceAnomalyDetector"
    }

    fn rule_id(&self) -> &str {
        &self.config.rule_id
    }

    fn enabled(&self) -> bool {
        self.config.enabled
    }

    async fn evaluate(&self, signals: &SignalStream) -> Result<Vec<Alert>, DetectorError> {
        let mut alerts = Vec::new();

        // Collect RFID scan events by device.
        let mut scans_by_device: HashMap<&str, HashSet<String>> = HashMap::new();
        let mut transit_start_by_device: HashMap<&str, chrono::DateTime<chrono::Utc>> =
            HashMap::new();
        let mut latest_time: HashMap<&str, chrono::DateTime<chrono::Utc>> = HashMap::new();

        for signal in signals {
            if signal.signal_type == "rfid_scan" {
                let tag = signal.payload["rfid_tag"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                scans_by_device
                    .entry(signal.device_id.as_str())
                    .or_default()
                    .insert(tag);
            }
            if signal.signal_type == "custody_transition"
                && signal.payload["state"].as_str() == Some("in_transit")
            {
                transit_start_by_device
                    .entry(signal.device_id.as_str())
                    .or_insert(signal.received_at);
            }
            let t = latest_time
                .entry(signal.device_id.as_str())
                .or_insert(signal.received_at);
            if signal.received_at > *t {
                *t = signal.received_at;
            }
        }

        for (device_id, expected_tags) in &self.expected_rfid {
            let Some(&transit_start) = transit_start_by_device.get(device_id.as_str()) else {
                continue;
            };
            let Some(&last_seen) = latest_time.get(device_id.as_str()) else {
                continue;
            };
            let elapsed = (last_seen - transit_start).num_minutes();
            if elapsed < self.config.absence_threshold_minutes {
                continue;
            }

            let seen_tags = scans_by_device
                .get(device_id.as_str())
                .cloned()
                .unwrap_or_default();
            let missing: HashSet<&String> = expected_tags.difference(&seen_tags).collect();
            if !missing.is_empty() {
                alerts.push(
                    Alert::new(
                        &self.config.rule_id,
                        "rfid_absence_anomaly",
                        AlertSeverity::Warn,
                        serde_json::json!({
                            "missing_rfid_tags": missing.into_iter().collect::<Vec<_>>(),
                            "elapsed_minutes": elapsed,
                            "threshold_minutes": self.config.absence_threshold_minutes,
                        }),
                    )
                    .with_device(device_id),
                );
            }
        }
        Ok(alerts)
    }
}
