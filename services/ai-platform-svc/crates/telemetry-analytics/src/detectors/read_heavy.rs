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

//! ReadHeavyAnomalyDetector: officer session with unusually high voter-record reads
//! without corresponding writes (potential data exfiltration pattern).
//!
//! Fires Warn alert when read_count > threshold AND read/write ratio > ratio_threshold.
//!
//! Reference: Insider threat detection via access pattern analysis
//! (Cappelli et al., "The CERT Guide to Insider Threats", 2012).
//!
//! Severity threshold: configurable read count + ratio (default: 100 reads, ratio 10:1).

use crate::detector::{Alert, AlertSeverity, Detector, DetectorError, SignalStream};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for ReadHeavyAnomalyDetector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadHeavyConfig {
    pub rule_id: String,
    /// Minimum absolute number of reads to consider as suspicious.
    pub min_read_count: u64,
    /// Minimum read-to-write ratio to consider as suspicious.
    pub read_write_ratio_threshold: f64,
    pub enabled: bool,
}

impl Default for ReadHeavyConfig {
    fn default() -> Self {
        Self {
            rule_id: "read_heavy_anomaly".into(),
            min_read_count: 100,
            read_write_ratio_threshold: 10.0,
            enabled: true,
        }
    }
}

pub struct ReadHeavyAnomalyDetector {
    config: ReadHeavyConfig,
}

impl ReadHeavyAnomalyDetector {
    pub fn new(config: ReadHeavyConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Detector for ReadHeavyAnomalyDetector {
    fn name(&self) -> &str {
        "ReadHeavyAnomalyDetector"
    }

    fn rule_id(&self) -> &str {
        &self.config.rule_id
    }

    fn enabled(&self) -> bool {
        self.config.enabled
    }

    async fn evaluate(&self, signals: &SignalStream) -> Result<Vec<Alert>, DetectorError> {
        let mut alerts = Vec::new();

        // Accumulate reads and writes per (officer_id, session_id).
        let mut reads: HashMap<(String, String), u64> = HashMap::new();
        let mut writes: HashMap<(String, String), u64> = HashMap::new();
        let mut device_map: HashMap<(String, String), String> = HashMap::new();

        for signal in signals {
            if signal.signal_type != "data_access" {
                continue;
            }
            let officer_id = match signal.payload["officer_id"].as_str() {
                Some(v) => v.to_string(),
                None => continue,
            };
            let session_id = match signal.payload["session_id"].as_str() {
                Some(v) => v.to_string(),
                None => continue,
            };
            let op = signal.payload["operation"].as_str().unwrap_or("");
            let key = (officer_id.clone(), session_id.clone());

            device_map.entry(key.clone()).or_insert_with(|| signal.device_id.clone());

            match op {
                "read" => {
                    *reads.entry(key).or_insert(0) += 1;
                }
                "write" => {
                    *writes.entry(key).or_insert(0) += 1;
                }
                _ => {}
            }
        }

        for ((officer_id, session_id), read_count) in &reads {
            if *read_count < self.config.min_read_count {
                continue;
            }
            let write_count = writes
                .get(&(officer_id.clone(), session_id.clone()))
                .copied()
                .unwrap_or(0);
            let ratio = if write_count == 0 {
                f64::INFINITY
            } else {
                *read_count as f64 / write_count as f64
            };
            if ratio < self.config.read_write_ratio_threshold {
                continue;
            }
            let device_id = device_map
                .get(&(officer_id.clone(), session_id.clone()))
                .cloned()
                .unwrap_or_default();

            alerts.push(
                Alert::new(
                    &self.config.rule_id,
                    "read_heavy_anomaly",
                    AlertSeverity::Warn,
                    serde_json::json!({
                        "officer_id": officer_id,
                        "session_id": session_id,
                        "read_count": read_count,
                        "write_count": write_count,
                        "read_write_ratio": ratio,
                        "threshold_reads": self.config.min_read_count,
                        "threshold_ratio": self.config.read_write_ratio_threshold,
                    }),
                )
                .with_officer(officer_id)
                .with_device(device_id),
            );
        }
        Ok(alerts)
    }
}
