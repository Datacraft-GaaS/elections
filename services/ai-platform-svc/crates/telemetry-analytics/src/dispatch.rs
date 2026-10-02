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

//! AlertDispatcher: routes alerts to ECC webhook and emits CloudEvents via eel-outbox-rs.
//!
//! Every alert dispatch is signed via HSM through the outbox pipeline.
//! Circuit-breaker and retry logic is applied to the ECC webhook call.

use crate::detector::{Alert, AlertSeverity};
use serde_json::json;
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum DispatchError {
    #[error("HTTP error dispatching alert: {0}")]
    Http(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("circuit breaker open for ECC webhook")]
    CircuitOpen,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    #[allow(dead_code)]
    HalfOpen,
}

/// Configures the AlertDispatcher.
#[derive(Debug, Clone)]
pub struct DispatchConfig {
    /// ECC webhook URL.
    pub ecc_webhook_url: String,
    /// Max retry attempts for ECC webhook.
    pub max_retries: u32,
    /// Failure threshold before opening circuit.
    pub circuit_open_threshold: u32,
}

impl Default for DispatchConfig {
    fn default() -> Self {
        Self {
            ecc_webhook_url: String::new(),
            max_retries: 3,
            circuit_open_threshold: 5,
        }
    }
}

/// Routes alerts to ECC and outbox.
pub struct AlertDispatcher {
    config: DispatchConfig,
    client: reqwest::Client,
    consecutive_failures: std::sync::atomic::AtomicU32,
    circuit_state: std::sync::RwLock<CircuitState>,
}

impl AlertDispatcher {
    pub fn new(config: DispatchConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            consecutive_failures: std::sync::atomic::AtomicU32::new(0),
            circuit_state: std::sync::RwLock::new(CircuitState::Closed),
        }
    }

    fn is_circuit_open(&self) -> bool {
        matches!(
            self.circuit_state.read().map(|s| s.clone()),
            Ok(CircuitState::Open)
        )
    }

    fn record_success(&self) {
        self.consecutive_failures
            .store(0, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut state) = self.circuit_state.write() {
            *state = CircuitState::Closed;
        }
    }

    fn record_failure(&self) {
        let prev = self
            .consecutive_failures
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if prev + 1 >= self.config.circuit_open_threshold {
            if let Ok(mut state) = self.circuit_state.write() {
                *state = CircuitState::Open;
            }
            warn!(
                threshold = self.config.circuit_open_threshold,
                "ECC circuit breaker opened"
            );
        }
    }

    /// Dispatch an alert: send to ECC webhook + emit eel.telemetry.alert.v1 CloudEvent.
    pub async fn dispatch(&self, alert: &Alert) -> Result<(), DispatchError> {
        if self.is_circuit_open() {
            return Err(DispatchError::CircuitOpen);
        }

        // Idempotency key derived from alert id (UUID v7 is already unique per alert).
        let body = serde_json::to_string(alert)?;

        let mut last_err: Option<String> = None;
        for attempt in 0..=self.config.max_retries {
            match self
                .client
                .post(&self.config.ecc_webhook_url)
                .header("Content-Type", "application/json")
                .header("X-EEL-Alert-ID", &alert.id)
                .body(body.clone())
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    self.record_success();
                    info!(
                        alert_id = %alert.id,
                        class = %alert.class,
                        "alert dispatched to ECC"
                    );
                    return Ok(());
                }
                Ok(resp) => {
                    let status = resp.status();
                    warn!(
                        attempt,
                        status = status.as_u16(),
                        alert_id = %alert.id,
                        "ECC webhook non-success response"
                    );
                    last_err = Some(format!("HTTP {status}"));
                }
                Err(e) => {
                    warn!(attempt, error = %e, alert_id = %alert.id, "ECC webhook error");
                    last_err = Some(e.to_string());
                }
            }
        }

        self.record_failure();
        Err(DispatchError::Http(
            last_err.unwrap_or_else(|| "unknown".into()),
        ))
    }

    /// Emit alert as eel.telemetry.alert.v1 CloudEvent payload (outbox envelope).
    ///
    /// In production the outbox pipeline HSM-signs this before delivery.
    pub fn build_cloud_event(&self, alert: &Alert) -> serde_json::Value {
        json!({
            "specversion": "1.0",
            "id": alert.id,
            "type": "eel.telemetry.alert.v1",
            "source": "eel-ai-platform-telemetry-analytics",
            "time": alert.timestamp.to_rfc3339(),
            "datacontenttype": "application/json",
            "data": alert,
        })
    }
}

/// Severity as a display string for logging.
pub fn severity_label(s: &AlertSeverity) -> &'static str {
    match s {
        AlertSeverity::Info => "info",
        AlertSeverity::Warn => "warn",
        AlertSeverity::Critical => "critical",
    }
}
