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

//! Clone detection and binary integrity for the field apps.
//!
//! `report_clone` minted a `report_id` and returned it, storing nothing and
//! reaching no conclusion: a filed clone report that always reports the same
//! shape tells an investigator nothing.
//!
//! `binary_integrity_check` returned **`passed: true` for every app** with
//! `failure_reason: None`. That is an attestation that a build is legitimate, and
//! it was made unconditionally. Verifying a real attestation needs the platform
//! key attestation service, which is not in this workspace, so the honest outcome
//! is a recorded event that says the check could not be performed.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use sha2::{Digest, Sha256};
use tracing::info;
use uuid::Uuid;

use crate::models::{BinaryIntegrityEvent, BinaryIntegrityRequest, CloneReportRequest};
use crate::verify_store::SharedVerifyStore;

/// A problem+json response.
fn problem(status: StatusCode, detail: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail.into(),
        })),
    )
        .into_response()
}

/// Whether a package name looks like a rebrand of a known one.
///
/// A real check needs the published package set, which this service does not hold.
/// What it can do is report the finding the request itself carries, and say that
/// no independent determination was made.
fn signature_findings(package_name: &str, reported_signature: &str) -> Vec<String> {
    let mut findings = Vec::new();
    if reported_signature.trim().is_empty() {
        findings.push(
            "no signing signature was supplied, so the build cannot be attributed".to_string(),
        );
    }
    // The app id and package name must agree in the last segment for a genuine
    // build; a mismatch is the common shape of a repackaged app.
    let app_tail = package_name
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !app_tail.is_empty() && !package_name.is_empty() && package_name.contains("..") {
        findings.push(format!("package name {package_name} has an empty segment"));
    }
    findings
}

/// File a clone report, recording it with a digest over its content.
pub async fn report_clone(
    State(store): State<SharedVerifyStore>,
    Json(req): Json<CloneReportRequest>,
) -> axum::response::Response {
    info!(package_name = %req.package_name, platform = %req.platform, "clone_detect.report_clone");
    if req.package_name.trim().is_empty() || req.app_id.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "app_id and package_name are required",
        );
    }
    let findings = signature_findings(&req.package_name, &req.reported_signature);
    // The report digest covers the request, so two identical filings are
    // distinguishable from one re-filing.
    let mut h = Sha256::new();
    h.update(b"eel.mobile.clone-report.v1");
    for part in [
        &req.app_id,
        &req.package_name,
        &req.reported_signature,
        &req.platform,
    ] {
        h.update([0x1e]);
        h.update(part.as_bytes());
    }
    let record = store.file_clone(&req.package_name, &req.reported_signature, findings.clone());

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "report_id": record.report_id,
            "app_id": req.app_id,
            "package_name": req.package_name,
            "reported_signature": req.reported_signature,
            "platform": req.platform,
            "reported_at": Utc::now(),
            "findings": record.findings,
            "report_hash": record.report_hash,
            "determination": "reported, not determined",
            "blocking_reason": "an independent clone determination needs the published \
                                 package set and the platform signing-key attestation, neither \
                                 of which is in this workspace",
        })),
    )
        .into_response()
}

/// Record a binary-integrity check.
pub async fn binary_integrity_check(
    Json(req): Json<BinaryIntegrityRequest>,
) -> axum::response::Response {
    let event_id = Uuid::now_v7().to_string();
    info!(event_id, app_id = %req.app_id, "clone_detect.binary_integrity_check");
    if req.app_id.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "app_id is required");
    }
    // `passed` is not asserted. The previous handler returned `passed: true` for
    // every app with `failure_reason: None`, which is an unconditional attestation
    // that a build is legitimate.
    //
    // What can be checked here is that the request carries the material an
    // attestation would be made from: a build fingerprint and an attestation
    // token. Without a token there is nothing to attest to, and without a
    // fingerprint there is nothing to bind it to.
    let mut missing: Vec<&str> = Vec::new();
    if req.build_fingerprint.trim().is_empty() {
        missing.push("build_fingerprint");
    }
    if req.attestation_token.trim().is_empty() {
        missing.push("attestation_token");
    }
    let event = BinaryIntegrityEvent {
        event_id,
        app_id: req.app_id,
        build_fingerprint: req.build_fingerprint.clone(),
        // Kept as supplied so the event is evidence of what was presented. The
        // token is not verified here and the event does not claim it was.
        attestation_token: req.attestation_token.clone(),
        passed: false,
        failure_reason: Some(if missing.is_empty() {
            "the attestation is present but cannot be verified here; verification needs the \
			 platform key attestation service, which is not in this workspace"
                .to_string()
        } else {
            format!(
                "the request carries no {}; there is nothing to attest to",
                missing.join(" or ")
            )
        }),
        emitted_at: Utc::now(),
    };
    (StatusCode::ACCEPTED, Json(event)).into_response()
}
