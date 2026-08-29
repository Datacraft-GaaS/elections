//! Public contract handlers (dereg/appeal.yaml).

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeregAppeal {
    pub appeal_id: String,
    pub status: String,
    pub submitted_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AppealDecisionCreate {
    pub verdict: String,
    pub grounds: String,
    pub decided_by_hash: String,
}

#[derive(Debug, Serialize)]
pub struct AppealDecision {
    pub decision_id: String,
    pub reinstated: bool,
    pub decided_at: String,
}

#[derive(Debug, Deserialize)]
pub struct DeregUssdRequest {
    pub session_id: String,
    pub msisdn_hash: String,
    pub text: String,
    pub service_code: String,
}

#[derive(Debug, Serialize)]
pub struct DeregUssdResponse {
    pub response_text: String,
    pub action: String,
}

fn page(items: &serde_json::Value) -> serde_json::Value {
    eel_spec_helpers_rs::page_flat_value(items)
}

/// GET /api/v1/dereg/appeals — list deregistration appeals.
pub async fn list_dereg_appeals(State(_state): State<AppState>) -> impl IntoResponse {
    Json(page(&serde_json::json!([])))
}

/// GET /api/v1/dereg/appeals/{appeal_id} — fetch one appeal.
pub async fn get_dereg_appeal(
    State(_state): State<AppState>,
    axum::extract::Path(appeal_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "detail": format!("appeal {appeal_id} not found") })),
    )
}

/// POST /api/v1/dereg/appeals/{appeal_id}/decisions — record an appeal decision.
pub async fn decide_dereg_appeal(
    State(_state): State<AppState>,
    axum::extract::Path(appeal_id): axum::extract::Path<String>,
    Json(req): Json<AppealDecisionCreate>,
) -> impl IntoResponse {
    let _ = appeal_id;
    (
        StatusCode::CREATED,
        Json(AppealDecision {
            decision_id: uuid::Uuid::now_v7().to_string(),
            reinstated: req.verdict == "reinstate",
            decided_at: chrono::Utc::now().to_rfc3339(),
        }),
    )
}

/// POST /api/v1/dereg/ussd/sessions — handle a dereg USSD session.
pub async fn handle_dereg_ussd_session(
    State(_state): State<AppState>,
    Json(_req): Json<DeregUssdRequest>,
) -> impl IntoResponse {
    Json(DeregUssdResponse {
        response_text: "END Service unavailable".to_string(),
        action: "end".to_string(),
    })
}


/// Composes the public dereg contract router.
pub fn router() -> axum::Router<crate::state::AppState> {
	axum::Router::new()
		.route("/api/v1/dereg/appeals", axum::routing::get(list_dereg_appeals))
		.route("/api/v1/dereg/appeals/{appeal_id}", axum::routing::get(get_dereg_appeal))
		.route("/api/v1/dereg/appeals/{appeal_id}/decisions", axum::routing::post(decide_dereg_appeal))
		.route("/api/v1/dereg/ussd/sessions", axum::routing::post(handle_dereg_ussd_session))
}
