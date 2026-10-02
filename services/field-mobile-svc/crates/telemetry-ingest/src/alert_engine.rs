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

//! Rule engine that evaluates incoming signals and raises alerts.
//!
//! Rules:
//!   drop            - class == drop
//!   battery_low     - battery_pct <= 10 (from battery signal)
//!   geo_fence_breach - location signal with out-of-bounds flag
//!   login_time_anomaly - login_logout at anomalous hour (before 06:00 or after 20:00 UTC)
//!   read_heavy      - class == read with payload.read_count > 500

use chrono::{DateTime, Timelike, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::{AlertClass, Severity, TelemetryAlert, TelemetrySignal};

/// A candidate alert built by the rule engine (not yet persisted).
#[derive(Debug, Clone)]
pub struct AlertCandidate {
    pub device_id: Uuid,
    pub class: AlertClass,
    pub severity: Severity,
    pub detected_at: DateTime<Utc>,
    pub triggering_signal: Option<Uuid>,
    pub payload: Value,
}

impl AlertCandidate {
    /// Convert to a `TelemetryAlert` domain row with a fresh UUID7 ID.
    pub fn into_alert(self) -> TelemetryAlert {
        TelemetryAlert {
            alert_id: Uuid::now_v7(),
            device_id: self.device_id,
            class: self.class,
            severity: self.severity,
            detected_at: self.detected_at,
            triggering_signal: self.triggering_signal,
            payload: self.payload,
            ecc_dispatched: false,
            ecc_dispatched_at: None,
            resolved_at: None,
            resolution_note: None,
        }
    }
}

/// Evaluate all alert rules against a single signal.
///
/// Returns zero or more alert candidates. Caller persists and dispatches them.
pub fn evaluate(signal: &TelemetrySignal) -> Vec<AlertCandidate> {
    let mut candidates = Vec::new();
    let now = signal.ingested_at;

    // Rule: drop signal.
    if signal.class == crate::domain::SignalClass::Drop {
        candidates.push(AlertCandidate {
            device_id: signal.device_id,
            class: AlertClass::Drop,
            severity: Severity::Critical,
            detected_at: now,
            triggering_signal: Some(signal.signal_id),
            payload: serde_json::json!({"signal_class": "drop"}),
        });
    }

    // Rule: battery_low (battery_pct <= 10).
    if signal.class == crate::domain::SignalClass::Battery {
        if let Some(pct) = signal.payload.get("battery_pct").and_then(Value::as_i64) {
            if pct <= 10 {
                candidates.push(AlertCandidate {
                    device_id: signal.device_id,
                    class: AlertClass::BatteryLow,
                    severity: Severity::Warn,
                    detected_at: now,
                    triggering_signal: Some(signal.signal_id),
                    payload: serde_json::json!({"battery_pct": pct}),
                });
            }
        }
    }

    // Rule: geo_fence_breach (location signal with out_of_bounds flag).
    if signal.class == crate::domain::SignalClass::Location {
        if signal
            .payload
            .get("out_of_bounds")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            candidates.push(AlertCandidate {
                device_id: signal.device_id,
                class: AlertClass::GeoFenceBreach,
                severity: Severity::Critical,
                detected_at: now,
                triggering_signal: Some(signal.signal_id),
                payload: signal.payload.clone(),
            });
        }
    }

    // Rule: login_time_anomaly (login outside 06:00-20:00 UTC).
    if signal.class == crate::domain::SignalClass::LoginLogout {
        let hour = signal.occurred_at.hour();
        if hour < 6 || hour >= 20 {
            candidates.push(AlertCandidate {
                device_id: signal.device_id,
                class: AlertClass::LoginTimeAnomaly,
                severity: Severity::Warn,
                detected_at: now,
                triggering_signal: Some(signal.signal_id),
                payload: serde_json::json!({"occurred_at_hour_utc": hour}),
            });
        }
    }

    // Rule: read_heavy (read signal with read_count > 500).
    if signal.class == crate::domain::SignalClass::Read {
        if let Some(count) = signal.payload.get("read_count").and_then(Value::as_i64) {
            if count > 500 {
                candidates.push(AlertCandidate {
                    device_id: signal.device_id,
                    class: AlertClass::ReadHeavy,
                    severity: Severity::Info,
                    detected_at: now,
                    triggering_signal: Some(signal.signal_id),
                    payload: serde_json::json!({"read_count": count}),
                });
            }
        }
    }

    candidates
}

/// Classify severity for a given alert class (used when overriding rule default).
pub fn classify_severity(class: &AlertClass) -> Severity {
    match class {
        AlertClass::Drop
        | AlertClass::GeoFenceBreach
        | AlertClass::SessionHijack
        | AlertClass::ChassisTamper => Severity::Critical,
        AlertClass::BatteryLow
        | AlertClass::LoginTimeAnomaly
        | AlertClass::RfidAbsence
        | AlertClass::CoordinatedAnomaly => Severity::Warn,
        AlertClass::ReadHeavy => Severity::Info,
    }
}
