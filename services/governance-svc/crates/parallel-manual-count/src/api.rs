use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::Value;
use std::sync::Arc;

use crate::divergence::compute_divergence;
use crate::ingest::{ManualCountStore, ManualCountSubmission};
use crate::plan::SamplePlan;

#[derive(Clone)]
pub struct AppState {
    pub plan: Arc<SamplePlan>,
    pub store: Arc<ManualCountStore>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/parallel-count/plan", get(get_plan))
        .route(
            "/v1/parallel-count/stations/:id/submit",
            post(submit_station),
        )
        .route("/v1/parallel-count/divergence", get(get_divergence))
        .with_state(state)
}

async fn get_plan(State(state): State<AppState>) -> Json<Value> {
    Json(serde_json::to_value(state.plan.as_ref()).unwrap_or(Value::Null))
}

async fn submit_station(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut submission): Json<ManualCountSubmission>,
) -> (StatusCode, Json<Value>) {
    // Enforce the path id takes precedence.
    submission.station_id = id;

    match state.store.submit(submission).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "status": "accepted" })),
        ),
        Err(msg) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": msg })),
        ),
    }
}

async fn get_divergence(State(state): State<AppState>) -> Json<Value> {
    // Divergence requires platform tallies; return report over current submissions with empty tallies.
    let submissions = state.store.all().await;
    let report = compute_divergence(&submissions, &[]);
    Json(serde_json::to_value(&report).unwrap_or(Value::Null))
}
