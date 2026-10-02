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

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::models::*;
use crate::spec;

use crate::models::RecordTransmissionRequest;
use crate::state::SharedState;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

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

/// The SHA-256 of the canonical JSON, prefixed the way the rest of the tree
/// writes payload hashes.
fn payload_hash(value: &serde_json::Value) -> String {
    let canonical = serde_json::to_vec(value).expect("Value is JSON-serialisable");
    let mut h = sha2::Sha256::new();
    use sha2::Digest;
    h.update(&canonical);
    format!("sha256:{}", hex::encode(h.finalize()))
}

pub async fn list_transmissions(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_transmissions");
    let guard = state.transmissions_legacy.lock().expect("lock poisoned");
    let transmissions: Vec<TransmissionRecord> = guard.clone();
    drop(guard);
    Json(serde_json::json!({ "transmissions": transmissions }))
}

pub async fn record_transmission(
    State(state): State<SharedState>,
    Json(req): Json<RecordTransmissionRequest>,
) -> axum::response::Response {
    info!(event_type = %req.event_type, sender = %req.sender, "record_transmission");
    if req.event_type.trim().is_empty()
        || req.sender.trim().is_empty()
        || req.recipient.trim().is_empty()
    {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "event_type, sender and recipient are required",
        );
    }
    // The caller may supply a payload_hash for a body this service never saw.
    // When they do not, the hash is computed over the payload actually received,
    // so the stored record is self-consistent rather than carrying an empty or
    // caller-asserted digest.
    let payload_hash = match req.payload_hash.filter(|h| !h.trim().is_empty()) {
        Some(h) => h,
        None => payload_hash(&serde_json::json!({
            "event_type": req.event_type,
            "sender": req.sender,
            "recipient": req.recipient,
            "payload": req.payload,
        })),
    };
    let record = TransmissionRecord {
        record_id: uuid::Uuid::now_v7().to_string(),
        event_type: req.event_type,
        sender: req.sender,
        recipient: req.recipient,
        transmitted_at: chrono::Utc::now().to_rfc3339(),
        payload_hash,
        legal_basis: req.legal_basis,
    };
    let verdicts = state.record_and_evaluate(record.clone());
    (StatusCode::CREATED, Json(serde_json::json!({
        "transmission": record,
        "verdicts": verdicts,
    })))
    .into_response()
}

pub async fn get_verdict(
    State(state): State<SharedState>,
    Path(record_id): Path<String>,
) -> axum::response::Response {
    info!(record_id, "get_verdict");
    let known = state
        .transmissions_legacy
        .lock()
        .expect("lock poisoned")
        .iter()
        .any(|r| r.record_id == record_id);
    if !known {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no transmission record with id {record_id}"),
        );
    }
    let verdicts = state.verdicts_for(&record_id);
    Json(serde_json::json!({ "record_id": record_id, "verdicts": verdicts })).into_response()
}

pub async fn list_verdicts(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_verdicts");
    let guard = state.verdicts.lock().expect("lock poisoned");
    let verdicts: Vec<ComplianceVerdict> = guard.clone();
    drop(guard);
    Json(serde_json::json!({
        "verdicts": verdicts,
        "chain_head": crate::rules::chain_head(&verdicts),
    }))
}

pub async fn party_summary(
    State(state): State<SharedState>,
    Path(party_id): Path<String>,
) -> axum::response::Response {
    info!(party_id, "party_summary");
    if !state.party_exists(&party_id) {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no party named {party_id} has transmitted anything"),
        );
    }
    let records = state.records_for_party(&party_id);
    let chain = state.verdicts.lock().expect("lock poisoned").clone();
    let refs: Vec<&TransmissionRecord> = records.iter().collect();
    let summary = crate::rules::summarise_party(&party_id, &party_id, &refs, &chain);
    (StatusCode::OK, Json(summary)).into_response()
}

pub async fn generate_report(State(state): State<SharedState>) -> axum::response::Response {
    info!("generate_report");
    let records = state.transmissions_legacy.lock().expect("lock poisoned").clone();
    let chain = state.verdicts.lock().expect("lock poisoned").clone();

    // The report is derived, not asserted: it counts what is actually stored and
    // says so in `scope`. It previously answered 202 with no body, which left no
    // way to tell a completed audit from one that had not run.
    let total = records.len();
    let failed: usize = chain.iter().filter(|v| !v.passed).count();
    let passed = chain.iter().filter(|v| v.passed).count();
    let unevaluated = records
        .iter()
        .filter(|r| !chain.iter().any(|v| v.record_id == r.record_id))
        .count();
    let mut findings: Vec<String> = Vec::new();
    for r in &records {
        let rec_verdicts: Vec<&ComplianceVerdict> =
            chain.iter().filter(|v| v.record_id == r.record_id).collect();
        if rec_verdicts.is_empty() {
            findings.push(format!(
                "record {} has no verdict: it was stored but never evaluated",
                r.record_id
            ));
            continue;
        }
        for v in rec_verdicts.iter().filter(|v| !v.passed) {
            findings.push(format!(
                "record {} failed {} ({})",
                r.record_id, v.rule_id, v.section_ref
            ));
        }
    }
    let body = serde_json::json!({
        "report_id": format!("audit-{}", &chain.first().map(|v| &v.verdict_hash[..8]).unwrap_or("empty")),
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "scope": "all recorded transmissions in this process's store",
        "compliance_summary": serde_json::json!({
            "transmissions": total,
            "verdicts": chain.len(),
            "verdicts_passed": passed,
            "verdicts_failed": failed,
            "transmissions_without_verdicts": unevaluated,
            "chain_head": crate::rules::chain_head(&chain),
            "chain_verifies": crate::rules::verify_chain_with_records(&chain, &records).is_none(),
            // There is no append-only log here, so inclusion is never claimed.
            "tlog_inclusion_proven": false,
        }),
        "findings": findings,
    });
    (StatusCode::OK, Json(body)).into_response()
}
// ---------------------------------------------------------------------------
// Public contract endpoints (governance-legal-audit/audit.yaml)
// ---------------------------------------------------------------------------

/// GET /api/v1/governance/legal-audit/transmissions — list transmission legal
/// bases.
pub async fn list_transmission_legal_records(
    State(state): State<SharedState>,
) -> impl IntoResponse {
    info!("list_transmission_legal_records");
    let guard = state.transmissions.lock().expect("lock poisoned");
    let mut items: Vec<spec::TransmissionLegalBasis> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.form_id.cmp(&b.form_id));
    Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }))
}

/// GET /api/v1/governance/legal-audit/transmissions/{form_id} — fetch one
/// transmission's legal basis.
pub async fn get_transmission_legal_basis(
    State(state): State<SharedState>,
    Path(form_id): Path<String>,
) -> impl IntoResponse {
    info!(form_id, "get_transmission_legal_basis");
    let guard = state.transmissions.lock().expect("lock poisoned");
    match guard.get(&form_id) {
        Some(t) => Json(t.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "transmission not found"})),
        )
            .into_response(),
    }
}

/// POST /api/v1/governance/legal-audit/certifications — certify a transmission
/// step.
pub async fn certify_transmission(
    State(state): State<SharedState>,
    Json(req): Json<spec::TransmissionCertificationRequest>,
) -> impl IntoResponse {
    info!(form_id = %req.form_id, step = %req.step, "certify_transmission");
    let certification = spec::TransmissionCertification {
        certification_id: uuid::Uuid::now_v7().to_string(),
        form_id: req.form_id,
        step: req.step,
        legal_instrument: req.legal_instrument,
        officer_hash: "pending-auth".to_string(),
        certified_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut guard = state.certifications.lock().expect("lock poisoned");
    guard.insert(certification.certification_id.clone(), certification.clone());
    drop(guard);
    (StatusCode::CREATED, Json(certification)).into_response()
}

/// GET /api/v1/governance/legal-audit/reports/{report_id} — fetch a legal-audit
/// report.
pub async fn get_legal_audit_report(
    State(state): State<SharedState>,
    Path(report_id): Path<String>,
) -> impl IntoResponse {
    info!(report_id, "get_legal_audit_report");
    let guard = state.certifications.lock().expect("lock poisoned");
    let cert_count = guard.len();
    drop(guard);
    Json(spec::LegalAuditReport {
        report_id,
        generated_at: chrono::Utc::now().to_rfc3339(),
        scope: "transmission-chain".to_string(),
        compliance_summary: serde_json::json!({ "certifications": cert_count }),
        findings: vec![],
    })
    .into_response()
}
