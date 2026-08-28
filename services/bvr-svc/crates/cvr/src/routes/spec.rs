//! Public contract handlers (cvr/register.yaml).

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct VoterLookup {
    pub id_number_hash: String,
    pub station_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VoterLookupResult {
    pub registered: bool,
    pub polling_station_name: Option<String>,
    pub ward: Option<String>,
    pub constituency: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterStats {
    pub total_registered: i64,
    pub counties: i64,
    pub as_of: String,
}

#[derive(Debug, Deserialize)]
pub struct UssdSessionRequest {
    pub session_id: String,
    pub msisdn_hash: String,
    pub text: String,
    pub service_code: String,
}

#[derive(Debug, Serialize)]
pub struct UssdSessionResponse {
    pub response_text: String,
    pub action: String,
}

/// POST /api/v1/cvr/lookups — verify a voter's registration.
pub async fn voter_lookup(
    State(_state): State<AppState>,
    Json(_req): Json<VoterLookup>,
) -> impl IntoResponse {
    Json(VoterLookupResult {
        registered: false,
        polling_station_name: None,
        ward: None,
        constituency: None,
    })
}

/// GET /api/v1/cvr/register/snapshot — download the register snapshot.
pub async fn get_register_snapshot(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Json(serde_json::json!({ "detail": "no snapshot available" })))
}

/// GET /api/v1/cvr/register/stats — register statistics.
pub async fn get_register_stats(State(_state): State<AppState>) -> impl IntoResponse {
    Json(RegisterStats {
        total_registered: 0,
        counties: 0,
        as_of: chrono::Utc::now().to_rfc3339(),
    })
}

/// POST /api/v1/cvr/ussd/sessions — handle a USSD session.
pub async fn handle_ussd_session(
    State(_state): State<AppState>,
    Json(_req): Json<UssdSessionRequest>,
) -> impl IntoResponse {
    Json(UssdSessionResponse {
        response_text: "END Service unavailable".to_string(),
        action: "end".to_string(),
    })
}


/// Composes the public cvr contract router.
pub fn router() -> axum::Router<crate::state::AppState> {
	axum::Router::new()
		.route("/api/v1/cvr/lookups", axum::routing::post(voter_lookup))
		.route("/api/v1/cvr/register/snapshot", axum::routing::get(get_register_snapshot))
		.route("/api/v1/cvr/register/stats", axum::routing::get(get_register_stats))
		.route("/api/v1/cvr/ussd/sessions", axum::routing::post(handle_ussd_session))
}
