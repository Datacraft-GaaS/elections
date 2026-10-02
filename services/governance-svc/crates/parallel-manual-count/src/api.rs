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

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::divergence::{compute_divergence, PlatformTally};
use crate::ingest::{ManualCountStore, ManualCountSubmission};
use crate::plan::SamplePlan;

/// Shared mutable state: manual-count submissions plus the platform tallies
/// they are compared against.
#[derive(Debug, Default)]
pub struct PmcState {
    pub submissions: ManualCountStore,
    pub tallies: RwLock<HashMap<String, PlatformTally>>,
}

impl PmcState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub plan: Arc<SamplePlan>,
    pub state: Arc<PmcState>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/parallel-count/plan", get(get_plan))
        .route(
            "/v1/parallel-count/stations/:id/submit",
            post(submit_station),
        )
        .route("/v1/parallel-count/tallies", post(submit_tallies))
        .route("/v1/parallel-count/divergence", get(get_divergence))
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_plan(State(state): State<AppState>) -> Json<Value> {
    Json(serde_json::to_value(state.plan.as_ref()).unwrap_or(Value::Null))
}

async fn submit_station(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut submission): Json<ManualCountSubmission>,
) -> Response {
    // Enforce the path id takes precedence.
    submission.station_id = id;

    if !state.plan.contains_station(&submission.station_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "station not in sample plan",
                "station_id": submission.station_id
            })),
        )
            .into_response();
    }

    match state.state.submissions.submit(submission).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "status": "accepted" })),
        )
            .into_response(),
        Err(msg) => (StatusCode::CONFLICT, Json(serde_json::json!({ "error": msg })))
            .into_response(),
    }
}

#[derive(serde::Deserialize)]
struct TallyUpload {
    tallies: Vec<PlatformTally>,
}

/// Ingest platform tallies for sampled stations so divergence can be computed.
async fn submit_tallies(
    State(state): State<AppState>,
    Json(upload): Json<TallyUpload>,
) -> Response {
    let mut guard = state.state.tallies.write().await;
    let mut accepted = 0usize;
    for t in upload.tallies {
        if state.plan.contains_station(&t.station_id) {
            guard.insert(t.station_id.clone(), t);
            accepted += 1;
        }
    }
    drop(guard);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "accepted": accepted })),
    )
        .into_response()
}

async fn get_divergence(State(state): State<AppState>) -> Json<Value> {
    let submissions = state.state.submissions.all().await;
    let tallies: Vec<PlatformTally> = state.state.tallies.read().await.values().cloned().collect();
    let report = compute_divergence(&submissions, &tallies);
    Json(serde_json::to_value(&report).unwrap_or(Value::Null))
}
