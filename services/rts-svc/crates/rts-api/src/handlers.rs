use axum::{extract::Path, http::StatusCode, response::{IntoResponse, Json}};
use tracing::info;
use uuid::Uuid;
use crate::models::*;

pub async fn submit_form(Json(req): Json<Form34ARequest>) -> impl IntoResponse {
    info!(station_code=%req.station_code, "submit_form: received");
    (StatusCode::ACCEPTED, Json(FormAccepted { form_id: Uuid::new_v4(), status: "accepted".to_string(), icr_job_id: Some(Uuid::new_v4().to_string()) }))
}
pub async fn sms_fallback(Json(req): Json<SmsFallbackRequest>) -> impl IntoResponse {
    info!(device_id=%req.device_id, "sms_fallback: received");
    (StatusCode::ACCEPTED, Json(FormAccepted { form_id: Uuid::new_v4(), status: "accepted".to_string(), icr_job_id: None }))
}
pub async fn provision_token(Json(req): Json<TokenRequest>) -> impl IntoResponse {
    info!(device_id=%req.device_id, "provision_token");
    (StatusCode::OK, Json(TokenResponse { token: format!("rts-token-stub-{}", Uuid::new_v4()), expires_at: "2030-01-01T00:00:00Z".to_string() }))
}
pub async fn ingest_event(Json(_req): Json<EventIngestRequest>) -> impl IntoResponse {
    let event_id = Uuid::new_v4().to_string();
    (StatusCode::ACCEPTED, Json(EventIngestResponse { event_id, status: "queued".to_string() }))
}
pub async fn get_ack(Path(scan_id): Path<Uuid>) -> impl IntoResponse {
    Json(AckResponse { scan_id, status: "pending".to_string(), form_id: None })
}
pub async fn list_snapshots() -> impl IntoResponse {
    Json(serde_json::json!({ "snapshots": [] }))
}
pub async fn get_snapshot(Path((scope, code)): Path<(String, String)>) -> impl IntoResponse {
    Json(SnapshotResponse { scope, code, snapshot_id: None, published_at: None, form34c: None, merkle_root: None })
}
pub async fn get_form_proof(Path(form_id): Path<Uuid>) -> impl IntoResponse {
    Json(ProofResponse { form_id, merkle_proof: vec![], merkle_root: "0".repeat(64) })
}
pub async fn compile_form34b() -> impl IntoResponse {
    (StatusCode::ACCEPTED, Json(serde_json::json!({ "status": "compilation_queued" })))
}
pub async fn compile_form34c() -> impl IntoResponse {
    (StatusCode::ACCEPTED, Json(serde_json::json!({ "status": "compilation_queued" })))
}
pub async fn get_icr_job(Path(form_id): Path<Uuid>) -> impl IntoResponse {
    Json(IcrJobResult { job_id: Uuid::new_v4().to_string(), form_id, fields: vec![], status: "pending".to_string() })
}
pub async fn get_icr_result(Path(job_id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({ "job_id": job_id, "status": "pending", "fields": [] }))
}
pub async fn get_audit(Path(form_id): Path<Uuid>) -> impl IntoResponse {
    Json(serde_json::json!({ "form_id": form_id, "events": [] }))
}
pub async fn create_dispute(Json(req): Json<DisputeRequest>) -> impl IntoResponse {
    info!(form_id=%req.form_id, "create_dispute");
    (StatusCode::CREATED, Json(DisputeCreated { dispute_id: Uuid::new_v4(), status: "open".to_string() }))
}
pub async fn resolve_dispute(Path(id): Path<Uuid>, Json(_req): Json<DisputeResolveRequest>) -> impl IntoResponse {
    Json(DisputeResolved { dispute_id: id, status: "resolved".to_string() })
}
pub async fn health() -> impl IntoResponse { StatusCode::OK }
