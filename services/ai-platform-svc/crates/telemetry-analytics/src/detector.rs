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

//! Core `Detector` trait and shared types for all anomaly detectors.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Severity levels for generated alerts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warn,
    Critical,
}

/// An alert produced by a detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique alert identifier (UUID v7).
    pub id: String,
    /// Detector rule id that produced this alert.
    pub rule_id: String,
    /// Anomaly class label (e.g. "drop_at_scale", "geo_fence_breach").
    pub class: String,
    pub severity: AlertSeverity,
    pub device_id: Option<String>,
    pub officer_id: Option<String>,
    pub station_id: Option<String>,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl Alert {
    /// Construct a new alert with a fresh UUID v7 id.
    pub fn new(
        rule_id: impl Into<String>,
        class: impl Into<String>,
        severity: AlertSeverity,
        details: serde_json::Value,
    ) -> Self {
        Self {
            id: uuid7_str(),
            rule_id: rule_id.into(),
            class: class.into(),
            severity,
            device_id: None,
            officer_id: None,
            station_id: None,
            details,
            timestamp: Utc::now(),
        }
    }

    pub fn with_device(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }

    pub fn with_officer(mut self, officer_id: impl Into<String>) -> Self {
        self.officer_id = Some(officer_id.into());
        self
    }

    pub fn with_station(mut self, station_id: impl Into<String>) -> Self {
        self.station_id = Some(station_id.into());
        self
    }
}

/// A single telemetry signal item from `eel_telemetry_signals`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySignal {
    pub id: String,
    pub device_id: String,
    pub signal_type: String,
    pub payload: serde_json::Value,
    pub received_at: DateTime<Utc>,
}

/// A batch of signals passed to a detector for evaluation.
pub type SignalStream = Vec<TelemetrySignal>;

/// Error type for detector failures.
#[derive(Debug, Error)]
pub enum DetectorError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("external service call failed: {0}")]
    ExternalService(String),
    #[error("configuration error: {0}")]
    Config(String),
}

/// Every anomaly detector must implement this trait.
///
/// Detectors receive a batch of signals and return zero or more alerts.
/// The `evaluate` method is async to allow gRPC/HTTP calls to external services.
#[async_trait::async_trait]
pub trait Detector: Send + Sync {
    /// Human-readable name of this detector.
    fn name(&self) -> &str;

    /// Rule id used in generated alerts (matches `RuleRegistry` entry).
    fn rule_id(&self) -> &str;

    /// Evaluate the signal stream and return any alerts.
    async fn evaluate(&self, signals: &SignalStream) -> Result<Vec<Alert>, DetectorError>;

    /// Whether this detector is enabled (default: true; override to check config).
    fn enabled(&self) -> bool {
        true
    }
}

/// Generate a UUID v7 string.
pub(crate) fn uuid7_str() -> String {
    Uuid::now_v7().to_string()
}
