use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tracing::info;
use uuid::Uuid;
use crate::models::*;

pub async fn post_module(Json(req): Json<CourseModule>) -> impl IntoResponse {
    info!(title=%req.title, module_type=%req.module_type, "post_module");
    (StatusCode::CREATED, Json(req))
}

pub async fn get_modules() -> impl IntoResponse {
    Json(serde_json::json!({ "modules": [] }))
}

pub async fn submit_assessment(
    Path(id): Path<Uuid>,
    Json(req): Json<AssessmentSubmitReq>,
) -> impl IntoResponse {
    info!(module_id=%id, idempotency_key=%req.idempotency_key, "submit_assessment");
    Json(serde_json::json!({ "module_id": id, "idempotency_key": req.idempotency_key, "status": "submitted" }))
}

pub async fn get_certificate(Path(worker_id): Path<Uuid>) -> impl IntoResponse {
    Json(WorkerCertificate {
        id: Uuid::new_v4(),
        worker_id,
        certificate_hash: "sha256-stub".to_string(),
        qr_verification_url: format!("/v1/training/lms/certificates/verify/stub-token"),
        issued_at: "2027-08-09T06:00:00Z".to_string(),
        expires_at: "2028-08-09T06:00:00Z".to_string(),
        revoked: false,
    })
}

pub async fn verify_certificate_qr(Path(qr_token): Path<String>) -> impl IntoResponse {
    info!(qr_token, "verify_certificate_qr");
    Json(CertificationStatus {
        worker_id: Uuid::new_v4(),
        status: "CERTIFIED".to_string(),
    })
}

pub async fn revoke_certificate(Path(id): Path<Uuid>) -> impl IntoResponse {
    info!(certificate_id=%id, "revoke_certificate");
    Json(serde_json::json!({ "id": id, "revoked": true }))
}

pub async fn get_progress() -> impl IntoResponse {
    Json(serde_json::json!({ "total_workers": 0, "certified": 0, "refresher_pending": 0, "not_certified": 0 }))
}

pub async fn confirm_briefing(Path(worker_id): Path<Uuid>) -> impl IntoResponse {
    info!(worker_id=%worker_id, "confirm_briefing");
    Json(serde_json::json!({ "worker_id": worker_id, "briefing_confirmed": true }))
}

pub async fn consume_crms_registered(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    info!(party_id=?req.get("party_id"), "crms.party.registered -> enroll");
    Json(serde_json::json!({"enrolled":true,"correlation_key":"crms-party-"}))
}
