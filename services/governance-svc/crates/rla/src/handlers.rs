use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use tracing::info;

use crate::models::*;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn list_audits() -> impl IntoResponse {
    info!("list_audits");
    let audits: Vec<AuditRun> = vec![];
    Json(serde_json::json!({ "audits": audits }))
}

pub async fn get_audit(Path(run_id): Path<String>) -> impl IntoResponse {
    info!(run_id, "get_audit");
    Json(AuditRun { run_id, ..Default::default() })
}

pub async fn create_audit() -> impl IntoResponse {
    info!("create_audit");
    Json(AuditRun::default())
}

pub async fn list_samples(Path(run_id): Path<String>) -> impl IntoResponse {
    info!(run_id, "list_samples");
    let samples: Vec<SampledBallot> = vec![];
    Json(serde_json::json!({ "samples": samples }))
}

pub async fn get_sample(Path((run_id, ballot_id)): Path<(String, String)>) -> impl IntoResponse {
    info!(run_id, ballot_id, "get_sample");
    Json(SampledBallot { ballot_id, ..Default::default() })
}

pub async fn record_count(Path((run_id, ballot_id)): Path<(String, String)>) -> impl IntoResponse {
    info!(run_id, ballot_id, "record_count");
    StatusCode::NO_CONTENT
}

pub async fn get_report(Path(run_id): Path<String>) -> impl IntoResponse {
    info!(run_id, "get_report");
    Json(AuditReport { run_id, ..Default::default() })
}

pub async fn finalise_audit(Path(run_id): Path<String>) -> impl IntoResponse {
    info!(run_id, "finalise_audit");
    StatusCode::NO_CONTENT
}
