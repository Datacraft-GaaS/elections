//! Admin API: axum 0.8 routes for rule management and detector status.
//!
//! Routes:
//!   GET    /v1/analytics/rules                  - list all rules
//!   POST   /v1/analytics/rules                  - create/update a rule
//!   GET    /v1/analytics/rules/{id}              - get a rule
//!   DELETE /v1/analytics/rules/{id}             - delete a rule
//!   GET    /v1/analytics/detectors/{id}/status  - detector enabled state
//!   POST   /v1/analytics/replay                 - trigger backtest replay

use crate::rule_registry::{RuleEntry, RuleRegistry};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct ApiState {
    pub registry: Arc<RuleRegistry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplayRequest {
    pub detector_id: String,
    pub window_start: String,
    pub window_end: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub detector_id: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

impl ApiError {
    fn new(msg: impl Into<String>) -> Json<Self> {
        Json(Self { error: msg.into() })
    }
}

/// Build the admin router.
pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/v1/analytics/rules", get(list_rules).post(upsert_rule))
        .route(
            "/v1/analytics/rules/{id}",
            get(get_rule).delete(delete_rule),
        )
        .route("/v1/analytics/detectors/{id}/status", get(detector_status))
        .route("/v1/analytics/replay", post(replay))
        .with_state(state)
}

async fn list_rules(State(s): State<ApiState>) -> impl IntoResponse {
    Json(s.registry.all())
}

async fn get_rule(
    State(s): State<ApiState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    s.registry
        .get(&id)
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, ApiError::new("rule not found")))
}

async fn upsert_rule(
    State(s): State<ApiState>,
    Json(rule): Json<RuleEntry>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    s.registry
        .upsert(rule)
        .map(|_| StatusCode::OK)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, ApiError::new(e)))
}

async fn delete_rule(
    State(s): State<ApiState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    match s.registry.delete(&id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((StatusCode::NOT_FOUND, ApiError::new("rule not found"))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, ApiError::new(e))),
    }
}

async fn detector_status(
    State(s): State<ApiState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    s.registry
        .get(&id)
        .map(|r| {
            Json(StatusResponse {
                detector_id: id.clone(),
                enabled: r.enabled,
            })
        })
        .ok_or_else(|| (StatusCode::NOT_FOUND, ApiError::new("detector not found")))
}

async fn replay(
    State(_s): State<ApiState>,
    Json(req): Json<ReplayRequest>,
) -> impl IntoResponse {
    // In production this triggers the backtest harness asynchronously.
    // For now we acknowledge the request and return accepted.
    tracing::info!(
        detector = %req.detector_id,
        window_start = %req.window_start,
        window_end = %req.window_end,
        "replay requested"
    );
    StatusCode::ACCEPTED
}
