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

//! Tamper-alert creation, severity classification, and ECC dispatch.

use chrono::Utc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::domain::{CoverTamperAlert, Severity};
use crate::error::RfidError;

// ---------------------------------------------------------------------------
// Severity classification
// ---------------------------------------------------------------------------

/// Classify alert severity based on the tamper type observed.
///
/// Rules:
/// - `challenge_valid = false` (clone attempt) -> Critical
/// - `tamper_indicator = true` (cover lifted) -> Critical
/// - EPC mismatch observed at a transit checkpoint -> Critical
/// - Cover lifted at an allowed transition reader -> Warn
/// - Any other anomaly -> Info
pub fn classify_severity(tamper_indicator: bool, challenge_valid: bool, epc_mismatch: bool) -> Severity {
    if epc_mismatch || !challenge_valid || tamper_indicator {
        Severity::Critical
    } else {
        Severity::Warn
    }
}

// ---------------------------------------------------------------------------
// ECC dispatch
// ---------------------------------------------------------------------------

/// Configuration for ECC (Emergency Communications Centre) webhook.
#[derive(Debug, Clone)]
pub struct EccConfig {
    /// Base URL of the ECC webhook endpoint.
    pub webhook_url: String,
    /// Bearer token for the ECC API.
    pub bearer_token: String,
}

/// Payload sent to the ECC webhook.
#[derive(Debug, serde::Serialize)]
struct EccDispatchPayload<'a> {
    alert_id: &'a str,
    tag_id: &'a str,
    severity: &'a str,
    detected_at: &'a str,
    expected_state: Option<&'a str>,
    observed_state: Option<&'a str>,
}

/// Dispatch a tamper alert to the ECC webhook.
///
/// Returns `Ok(())` on success; `Err(EccDispatch)` on HTTP or connection error.
/// Non-blocking: uses `reqwest` async client.
pub async fn dispatch_to_ecc(
    client: &reqwest::Client,
    config: &EccConfig,
    alert: &CoverTamperAlert,
) -> Result<(), RfidError> {
    let payload = EccDispatchPayload {
        alert_id: &alert.alert_id.to_string(),
        tag_id: &alert.tag_id.to_string(),
        severity: alert.severity.as_str(),
        detected_at: &alert.detected_at.to_rfc3339(),
        expected_state: alert.expected_state.as_deref(),
        observed_state: alert.observed_state.as_deref(),
    };

    let resp = client
        .post(&config.webhook_url)
        .bearer_auth(&config.bearer_token)
        .json(&payload)
        .send()
        .await
        .map_err(|e| RfidError::EccDispatch(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(RfidError::EccDispatch(format!(
            "ECC webhook returned {status}"
        )));
    }

    info!(
        alert_id = %alert.alert_id,
        severity = %alert.severity,
        "ECC dispatch succeeded"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Alert builder
// ---------------------------------------------------------------------------

/// Request to raise a new tamper alert.
#[derive(Debug, Clone)]
pub struct RaiseAlertRequest {
    pub tag_id: Uuid,
    pub triggering_read: Option<Uuid>,
    pub expected_state: Option<String>,
    pub observed_state: Option<String>,
    pub severity: Severity,
}

/// Build a `CoverTamperAlert` from a raise request (no DB write here; see repository).
pub fn build_alert(req: RaiseAlertRequest) -> CoverTamperAlert {
    let alert_id = Uuid::now_v7();
    let severity = req.severity.clone();

    match &severity {
        Severity::Critical => error!(
            alert_id = %alert_id,
            tag_id = %req.tag_id,
            "CRITICAL tamper alert: cover lift or clone detected"
        ),
        Severity::Warn => warn!(
            alert_id = %alert_id,
            tag_id = %req.tag_id,
            "WARN tamper alert"
        ),
        Severity::Info => info!(
            alert_id = %alert_id,
            tag_id = %req.tag_id,
            "INFO tamper event"
        ),
    }

    CoverTamperAlert {
        alert_id,
        tag_id: req.tag_id,
        detected_at: Utc::now(),
        triggering_read: req.triggering_read,
        expected_state: req.expected_state,
        observed_state: req.observed_state,
        severity,
        ecc_dispatched: false,
        ecc_dispatched_at: None,
        resolved_at: None,
        resolution_note: None,
    }
}
