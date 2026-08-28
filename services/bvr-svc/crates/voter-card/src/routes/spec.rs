//! Public contract handlers (voter-card/issuance.yaml).

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterCardRequest {
    pub request_id: String,
    pub status: String,
    pub submitted_at: String,
}

#[derive(Debug, Deserialize)]
pub struct VoterCardRequestCreate {
    pub voter_id_hash: String,
    pub request_type: String,
    pub reason: String,
    pub preferred_collection_centre: String,
}

#[derive(Debug, Deserialize)]
pub struct CollectionConfirmation {
    pub collected_at: String,
    pub collection_centre_code: String,
    pub officer_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionCentre {
    pub centre_code: String,
    pub name: String,
    pub county_code: String,
    pub address: String,
    pub operating_hours: Option<String>,
    pub open: bool,
}

#[derive(Clone, Default)]
pub struct SpecState {
    pub requests: Arc<Mutex<HashMap<String, VoterCardRequest>>>,
    pub centres: Arc<Mutex<HashMap<String, CollectionCentre>>>,
}

/// POST /api/v1/voter-card/requests — request a voter card.
pub async fn request_voter_card(
    State(state): State<AppState>,
    Json(_req): Json<VoterCardRequestCreate>,
) -> impl IntoResponse {
    let request = VoterCardRequest {
        request_id: uuid::Uuid::now_v7().to_string(),
        status: "submitted".to_string(),
        submitted_at: chrono::Utc::now().to_rfc3339(),
    };
    let _ = state;
    (StatusCode::CREATED, Json(request))
}

/// GET /api/v1/voter-card/requests/{request_id} — fetch a request.
pub async fn get_voter_card_request(
    State(_state): State<AppState>,
    Path(request_id): Path<String>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "detail": format!("request {request_id} not found") })),
    )
}

/// POST /api/v1/voter-card/requests/{request_id}/collection-confirmation.
pub async fn confirm_voter_card_collection(
    State(_state): State<AppState>,
    Path(request_id): Path<String>,
    Json(req): Json<CollectionConfirmation>,
) -> impl IntoResponse {
    let _ = (request_id, req);
    Json(serde_json::json!({ "status": "collected" }))
}

/// GET /api/v1/voter-card/collection-centres — list collection centres.
pub async fn list_collection_centres(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({ "centres": [], "total": 0 }))
}


/// Composes the public voter-card contract router.
pub fn router() -> axum::Router<crate::state::AppState> {
	axum::Router::new()
		.route("/api/v1/voter-card/requests", axum::routing::post(request_voter_card))
		.route("/api/v1/voter-card/requests/{request_id}", axum::routing::get(get_voter_card_request))
		.route("/api/v1/voter-card/requests/{request_id}/collection-confirmation", axum::routing::post(confirm_voter_card_collection))
		.route("/api/v1/voter-card/collection-centres", axum::routing::get(list_collection_centres))
}
