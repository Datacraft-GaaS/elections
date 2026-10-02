// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
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

/// Run a detector over a historical window.
///
/// Previously logged the detector id and the window and answered 202, with the
/// comment "In production this triggers the backtest harness asynchronously. For
/// now we acknowledge the request and return accepted." So a replay was accepted
/// and never run, and the caller was left holding a 202 for a result that did not
/// exist.
///
/// Replaying a window means reading the stored event stream for that window and
/// running the detector over it. Neither the event store nor the backtest harness
/// is in this workspace, so the honest answer is a refusal that names them.
async fn replay(
    State(_s): State<ApiState>,
    Json(req): Json<ReplayRequest>,
) -> Response {
    // The window bounds are RFC 3339 strings on the contract. Checked here because
    // an unparseable window cannot be replayed and saying so is more useful than a
    // blanket refusal.
    if chrono::DateTime::parse_from_rfc3339(&req.window_start).is_err()
        || chrono::DateTime::parse_from_rfc3339(&req.window_end).is_err()
    {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::new("window_start and window_end must be RFC 3339 timestamps"),
        )
            .into_response();
    }
    if req.detector_id.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::new("detector_id is required"),
        )
            .into_response();
    }
    (
        StatusCode::NOT_IMPLEMENTED,
        ApiError::new(
            "replaying a window needs the stored event stream for that window and the \
             detector backtest harness, and neither is in this workspace. This handler \
             previously answered 202 Accepted, so a caller held a receipt for a replay \
             that was never run",
        ),
    )
        .into_response()
}
