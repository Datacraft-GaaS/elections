//! GeoFenceBreachDetector: kits outside their assigned station polygon during voting hours.
//!
//! Uses geo crate for polygon containment checks (rayon parallel evaluation per device).
//! Fires ECC dispatch on any device breach sustained for more than 30 seconds.
//!
//! Reference: OGC Simple Features Access - Part 1: Common Architecture (OGC 06-103r4)

use crate::detector::{Alert, AlertSeverity, Detector, DetectorError, SignalStream};
use async_trait::async_trait;
use geo::{Contains, Coord, Point, Polygon};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for GeoFenceBreachDetector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoFenceConfig {
    pub rule_id: String,
    /// Minimum sustained breach duration in seconds before firing.
    pub breach_threshold_secs: u64,
    pub enabled: bool,
}

impl Default for GeoFenceConfig {
    fn default() -> Self {
        Self {
            rule_id: "geo_fence_breach".into(),
            breach_threshold_secs: 30,
            enabled: true,
        }
    }
}

/// A station geofence polygon (simplified: bounding box of exterior ring).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationFence {
    pub station_id: String,
    /// Exterior ring as [lon, lat] pairs.
    pub exterior: Vec<[f64; 2]>,
}

impl StationFence {
    fn polygon(&self) -> Option<Polygon<f64>> {
        if self.exterior.len() < 3 {
            return None;
        }
        let coords: Vec<Coord<f64>> = self
            .exterior
            .iter()
            .map(|p| Coord { x: p[0], y: p[1] })
            .collect();
        Some(Polygon::new(coords.into(), vec![]))
    }

    pub fn contains_point(&self, lon: f64, lat: f64) -> bool {
        self.polygon()
            .map(|poly| poly.contains(&Point::new(lon, lat)))
            .unwrap_or(false)
    }
}

pub struct GeoFenceBreachDetector {
    config: GeoFenceConfig,
    /// Station fences indexed by station_id.
    fences: HashMap<String, StationFence>,
}

impl GeoFenceBreachDetector {
    pub fn new(config: GeoFenceConfig, fences: Vec<StationFence>) -> Self {
        let fences = fences
            .into_iter()
            .map(|f| (f.station_id.clone(), f))
            .collect();
        Self { config, fences }
    }
}

#[async_trait]
impl Detector for GeoFenceBreachDetector {
    fn name(&self) -> &str {
        "GeoFenceBreachDetector"
    }

    fn rule_id(&self) -> &str {
        &self.config.rule_id
    }

    fn enabled(&self) -> bool {
        self.config.enabled
    }

    async fn evaluate(&self, signals: &SignalStream) -> Result<Vec<Alert>, DetectorError> {
        let mut alerts = Vec::new();

        // Group location signals by device.
        let mut by_device: HashMap<&str, Vec<&crate::detector::TelemetrySignal>> = HashMap::new();
        for signal in signals {
            if signal.signal_type == "location" {
                by_device
                    .entry(signal.device_id.as_str())
                    .or_default()
                    .push(signal);
            }
        }

        for (device_id, dev_signals) in &by_device {
            // Find the assigned station for this device from the first signal.
            let Some(assigned_station) = dev_signals
                .first()
                .and_then(|s| s.payload["assigned_station_id"].as_str())
            else {
                continue;
            };

            let Some(fence) = self.fences.get(assigned_station) else {
                continue;
            };

            // Find signals where device is outside fence during voting hours.
            let mut breach_start: Option<chrono::DateTime<chrono::Utc>> = None;
            for sig in dev_signals.iter() {
                let lon = sig.payload["lon"].as_f64().unwrap_or(0.0);
                let lat = sig.payload["lat"].as_f64().unwrap_or(0.0);
                let during_voting = sig.payload["voting_active"].as_bool().unwrap_or(false);

                if !fence.contains_point(lon, lat) && during_voting {
                    if breach_start.is_none() {
                        breach_start = Some(sig.received_at);
                    }
                } else {
                    breach_start = None;
                }

                if let Some(start) = breach_start {
                    let duration = (sig.received_at - start).num_seconds() as u64;
                    if duration >= self.config.breach_threshold_secs {
                        alerts.push(
                            Alert::new(
                                &self.config.rule_id,
                                "geo_fence_breach",
                                AlertSeverity::Critical,
                                serde_json::json!({
                                    "lon": lon,
                                    "lat": lat,
                                    "assigned_station_id": assigned_station,
                                    "breach_duration_secs": duration,
                                }),
                            )
                            .with_device(*device_id)
                            .with_station(assigned_station),
                        );
                        breach_start = None; // Reset after firing.
                    }
                }
            }
        }
        Ok(alerts)
    }
}
