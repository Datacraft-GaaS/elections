use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, nfc::nfc};

#[derive(Debug, Serialize)]
pub struct VoterResponse {
    pub voter_id: String,
    pub full_name: String,
    pub county_code: String,
    pub polling_station_id: String,
    pub status: String,
    pub election_scope: String,
}

#[derive(Debug, Deserialize)]
pub struct StatusUpdateRequest {
    pub status: String,
    pub reason: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/bvr/voters/{voter_id}", get(get_voter))
        .route("/api/v1/bvr/voters/{voter_id}/status", patch(update_voter_status))
        .route("/api/v1/bvr/voters/{voter_id}/qr", get(get_voter_qr))
}

async fn get_voter(Path(voter_id): Path<String>) -> Result<Json<VoterResponse>, ApiError> {
    Ok(Json(VoterResponse {
        voter_id: voter_id.clone(),
        full_name: nfc("Placeholder Voter"),
        county_code: nfc("047"),
        polling_station_id: nfc("047-001-001"),
        status: "active".into(),
        election_scope: nfc("2027-general"),
    }))
}

async fn update_voter_status(
    Path(voter_id): Path<String>,
    Json(req): Json<StatusUpdateRequest>,
) -> Result<StatusCode, ApiError> {
    tracing::info!(voter_id = %voter_id, status = %req.status, "voter status update");
    Ok(StatusCode::NO_CONTENT)
}

async fn get_voter_qr(
    Path(voter_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({
        "voter_id": voter_id,
        "qr_data": format!("eel://bvr/voter/{voter_id}"),
    })))
}
