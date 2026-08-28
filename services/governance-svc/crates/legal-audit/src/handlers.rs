use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::models::*;
use crate::spec;
use crate::state::SharedState;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn list_transmissions() -> impl IntoResponse {
    info!("list_transmissions");
    let transmissions: Vec<TransmissionRecord> = vec![];
    Json(serde_json::json!({ "transmissions": transmissions }))
}

pub async fn record_transmission() -> impl IntoResponse {
    info!("record_transmission");
    Json(TransmissionRecord::default())
}

pub async fn get_verdict(Path(record_id): Path<String>) -> impl IntoResponse {
    info!(record_id, "get_verdict");
    Json(ComplianceVerdict { record_id, ..Default::default() })
}

pub async fn list_verdicts() -> impl IntoResponse {
    info!("list_verdicts");
    let verdicts: Vec<ComplianceVerdict> = vec![];
    Json(serde_json::json!({ "verdicts": verdicts }))
}

pub async fn party_summary(Path(party_id): Path<String>) -> impl IntoResponse {
    info!(party_id, "party_summary");
    Json(PartySummary { party_id, ..Default::default() })
}

pub async fn generate_report() -> impl IntoResponse {
    info!("generate_report");
    StatusCode::ACCEPTED
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
