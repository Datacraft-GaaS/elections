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

//! LoginTimeAnomalyDetector: officer login at unusual time-of-day for that officer
//! and station, based on a baseline learned over the first 14 days of operation.
//!
//! Reference: Anomaly detection via statistical Z-score on hourly login distribution
//! (Nelson 1990, IEEE Transactions on Reliability).
//!
//! Threshold: Z-score > 2.5 (configurable) triggers Warn alert.

use crate::detector::{Alert, AlertSeverity, Detector, DetectorError, SignalStream};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for LoginTimeAnomalyDetector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginTimeConfig {
    pub rule_id: String,
    /// Z-score threshold: logins deviating more than this standard deviations are anomalous.
    pub zscore_threshold: f64,
    pub enabled: bool,
}

impl Default for LoginTimeConfig {
    fn default() -> Self {
        Self {
            rule_id: "login_time_anomaly".into(),
            zscore_threshold: 2.5,
            enabled: true,
        }
    }
}

/// Baseline: mean and std-dev of login hour per (officer_id, station_id).
#[derive(Debug, Clone)]
pub struct LoginBaseline {
    pub mean_hour: f64,
    pub std_hour: f64,
}

impl LoginBaseline {
    pub fn from_samples(hours: &[f64]) -> Option<Self> {
        if hours.len() < 2 {
            return None;
        }
        let mean = hours.iter().sum::<f64>() / hours.len() as f64;
        let variance = hours.iter().map(|h| (h - mean).powi(2)).sum::<f64>()
            / (hours.len() - 1) as f64;
        let std = variance.sqrt();
        if std < f64::EPSILON {
            return None;
        }
        Some(Self { mean_hour: mean, std_hour: std })
    }

    pub fn zscore(&self, hour: f64) -> f64 {
        (hour - self.mean_hour).abs() / self.std_hour
    }
}

pub struct LoginTimeAnomalyDetector {
    config: LoginTimeConfig,
    /// Baselines keyed by (officer_id, station_id).
    baselines: HashMap<(String, String), LoginBaseline>,
}

impl LoginTimeAnomalyDetector {
    pub fn new(
        config: LoginTimeConfig,
        baselines: HashMap<(String, String), LoginBaseline>,
    ) -> Self {
        Self { config, baselines }
    }
}

#[async_trait]
impl Detector for LoginTimeAnomalyDetector {
    fn name(&self) -> &str {
        "LoginTimeAnomalyDetector"
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
            if signal.signal_type != "login" {
                continue;
            }
            let officer_id = match signal.payload["officer_id"].as_str() {
                Some(v) => v.to_string(),
                None => continue,
            };
            let station_id = match signal.payload["station_id"].as_str() {
                Some(v) => v.to_string(),
                None => continue,
            };
            let hour = signal.received_at.format("%H").to_string().parse::<f64>().unwrap_or(0.0);

            let key = (officer_id.clone(), station_id.clone());
            let Some(baseline) = self.baselines.get(&key) else {
                continue;
            };

            let zscore = baseline.zscore(hour);
            if zscore > self.config.zscore_threshold {
                alerts.push(
                    Alert::new(
                        &self.config.rule_id,
                        "login_time_anomaly",
                        AlertSeverity::Warn,
                        serde_json::json!({
                            "officer_id": officer_id,
                            "station_id": station_id,
                            "login_hour": hour,
                            "baseline_mean_hour": baseline.mean_hour,
                            "baseline_std_hour": baseline.std_hour,
                            "zscore": zscore,
                            "threshold": self.config.zscore_threshold,
                        }),
                    )
                    .with_officer(&officer_id)
                    .with_station(&station_id)
                    .with_device(signal.device_id.clone()),
                );
            }
        }
        Ok(alerts)
    }
}
