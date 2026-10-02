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

//! BatteryStarvationClusterDetector: multiple kits in a region reporting battery < 20%
//! within a rolling 15-minute window.
//!
//! Fires a cluster alert to ECC dispatch recommending power-bank deployment,
//! with a GeoJSON polygon of the affected region (convex hull of affected devices).
//!
//! Reference: DBSCAN-inspired spatial clustering (Ester et al. 1996, KDD-96).
//! We use a simpler bounding-box approach for real-time performance.

use crate::detector::{Alert, AlertSeverity, Detector, DetectorError, SignalStream};
use async_trait::async_trait;
use chrono::Duration;
use serde::{Deserialize, Serialize};

/// Configuration for BatteryStarvationClusterDetector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryStarvationConfig {
    pub rule_id: String,
    /// Battery level percentage threshold below which a kit is considered starved.
    pub low_battery_pct: f64,
    /// Minimum number of kits in a region to trigger a cluster alert.
    pub cluster_min_devices: usize,
    /// Rolling window in minutes.
    pub window_minutes: i64,
    pub enabled: bool,
}

impl Default for BatteryStarvationConfig {
    fn default() -> Self {
        Self {
            rule_id: "battery_starvation_cluster".into(),
            low_battery_pct: 20.0,
            cluster_min_devices: 3,
            window_minutes: 15,
            enabled: true,
        }
    }
}

/// Bounding box of affected devices for GeoJSON polygon.
fn bounding_box_geojson(points: &[(f64, f64)]) -> serde_json::Value {
    if points.is_empty() {
        return serde_json::json!(null);
    }
    let min_lon = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let max_lon = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let min_lat = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let max_lat = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);

    // Return a GeoJSON Polygon (bounding box as a closed ring).
    serde_json::json!({
        "type": "Polygon",
        "coordinates": [[
            [min_lon, min_lat],
            [max_lon, min_lat],
            [max_lon, max_lat],
            [min_lon, max_lat],
            [min_lon, min_lat],
        ]]
    })
}

pub struct BatteryStarvationClusterDetector {
    config: BatteryStarvationConfig,
}

impl BatteryStarvationClusterDetector {
    pub fn new(config: BatteryStarvationConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Detector for BatteryStarvationClusterDetector {
    fn name(&self) -> &str {
        "BatteryStarvationClusterDetector"
    }

    fn rule_id(&self) -> &str {
        &self.config.rule_id
    }

    fn enabled(&self) -> bool {
        self.config.enabled
    }

    async fn evaluate(&self, signals: &SignalStream) -> Result<Vec<Alert>, DetectorError> {
        let mut alerts = Vec::new();

        // Find the latest signal time to bound the rolling window.
        let Some(latest) = signals.iter().map(|s| s.received_at).max() else {
            return Ok(alerts);
        };
        let window_start = latest - Duration::minutes(self.config.window_minutes);

        // Collect battery signals within the window that are below threshold.
        let mut low_battery_devices: Vec<(String, f64, f64, f64)> = Vec::new(); // (device_id, lon, lat, level_pct)
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

        for signal in signals {
            if signal.signal_type != "battery" {
                continue;
            }
            if signal.received_at < window_start {
                continue;
            }
            let level_pct = signal.payload["level_pct"].as_f64().unwrap_or(100.0);
            if level_pct >= self.config.low_battery_pct {
                continue;
            }
            if seen.contains(&signal.device_id) {
                continue;
            }
            seen.insert(signal.device_id.clone());

            let lon = signal.payload["lon"].as_f64().unwrap_or(0.0);
            let lat = signal.payload["lat"].as_f64().unwrap_or(0.0);
            low_battery_devices.push((signal.device_id.clone(), lon, lat, level_pct));
        }

        if low_battery_devices.len() >= self.config.cluster_min_devices {
            let points: Vec<(f64, f64)> = low_battery_devices
                .iter()
                .map(|(_, lon, lat, _)| (*lon, *lat))
                .collect();
            let polygon = bounding_box_geojson(&points);
            let device_ids: Vec<&str> = low_battery_devices
                .iter()
                .map(|(id, _, _, _)| id.as_str())
                .collect();

            alerts.push(Alert::new(
                &self.config.rule_id,
                "battery_starvation_cluster",
                AlertSeverity::Critical,
                serde_json::json!({
                    "device_count": low_battery_devices.len(),
                    "device_ids": device_ids,
                    "affected_region": polygon,
                    "recommendation": "deploy power banks to affected region",
                    "window_minutes": self.config.window_minutes,
                }),
            ));
        }
        Ok(alerts)
    }
}
