use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use tracing::info;

use crate::models::*;

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
