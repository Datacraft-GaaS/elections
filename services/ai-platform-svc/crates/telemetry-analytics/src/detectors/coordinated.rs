//! CoordinatedDeviceAnomalyDetector: same anomaly class across N devices
//! within a T-minute window fires a Critical alert.
//!
//! Anomaly classes watched: drop, battery_low, geo_fence_breach, login_anomaly.
//!
//! Reference: Coordinated attack detection via temporal clustering
//! (Wagner & Plattner, "Entropy Based Worm and Anomaly Detection in Fast IP Networks", 2005).

use crate::detector::{Alert, AlertSeverity, Detector, DetectorError, SignalStream};
use async_trait::async_trait;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Configuration for CoordinatedDeviceAnomalyDetector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatedConfig {
    pub rule_id: String,
    /// Minimum number of distinct devices reporting the same class within the window.
    pub min_devices: usize,
    /// Rolling window in minutes.
    pub window_minutes: i64,
    /// Anomaly classes to watch for coordinated patterns.
    pub watched_classes: Vec<String>,
    pub enabled: bool,
}

impl Default for CoordinatedConfig {
    fn default() -> Self {
        Self {
            rule_id: "coordinated_device_anomaly".into(),
            min_devices: 3,
            window_minutes: 10,
            watched_classes: vec![
                "drop".into(),
                "battery_low".into(),
                "geo_fence_breach".into(),
                "login_anomaly".into(),
            ],
            enabled: true,
        }
    }
}

pub struct CoordinatedDeviceAnomalyDetector {
    config: CoordinatedConfig,
}

impl CoordinatedDeviceAnomalyDetector {
    pub fn new(config: CoordinatedConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Detector for CoordinatedDeviceAnomalyDetector {
    fn name(&self) -> &str {
        "CoordinatedDeviceAnomalyDetector"
    }

    fn rule_id(&self) -> &str {
        &self.config.rule_id
    }

    fn enabled(&self) -> bool {
        self.config.enabled
    }

    async fn evaluate(&self, signals: &SignalStream) -> Result<Vec<Alert>, DetectorError> {
        let mut alerts = Vec::new();
        let watched: HashSet<&str> = self.config.watched_classes.iter().map(|s| s.as_str()).collect();

        // Find the latest signal time to bound the rolling window.
        let Some(latest) = signals.iter().map(|s| s.received_at).max() else {
            return Ok(alerts);
        };
        let window_start = latest - Duration::minutes(self.config.window_minutes);

        // Accumulate (class -> set of device_ids) within window.
        let mut class_devices: HashMap<&str, HashSet<&str>> = HashMap::new();

        for signal in signals {
            if signal.received_at < window_start {
                continue;
            }
            let class = match signal.payload["anomaly_class"].as_str() {
                Some(c) if watched.contains(c) => c,
                _ => continue,
            };
            class_devices
                .entry(class)
                .or_default()
                .insert(signal.device_id.as_str());
        }

        for (class, devices) in &class_devices {
            if devices.len() >= self.config.min_devices {
                alerts.push(Alert::new(
                    &self.config.rule_id,
                    "coordinated_device_anomaly",
                    AlertSeverity::Critical,
                    serde_json::json!({
                        "anomaly_class": class,
                        "device_count": devices.len(),
                        "device_ids": devices.iter().collect::<Vec<_>>(),
                        "window_minutes": self.config.window_minutes,
                        "threshold_devices": self.config.min_devices,
                    }),
                ));
            }
        }
        Ok(alerts)
    }
}
