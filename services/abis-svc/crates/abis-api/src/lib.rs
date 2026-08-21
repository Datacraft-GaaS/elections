//! ABIS API library: router and shared types.

pub mod models;
pub mod routes;
pub mod state;

use std::sync::Arc;

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    Json,
    routing::{get, patch, post},
};

use models::HealthResponse;
use routes::{
    adjudication::{list_adjudication, resolve_adjudication},
    batch::submit_batch_dedup,
    match_routes::{get_match_result, match_11, match_1n},
    reports::get_reports,
};
use state::AppState;

async fn healthz(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<HealthResponse>) {
    (StatusCode::OK, Json(HealthResponse { ok: true }))
}

pub fn app(state: Arc<AppState>) -> Router {
    let abis_routes = Router::new()
        .route("/match/1n", post(match_1n))
        .route("/match/11", post(match_11))
        .route("/match/{request_id}", get(get_match_result))
        .route("/adjudication", get(list_adjudication))
        .route("/adjudication/{case_id}", patch(resolve_adjudication))
        .route("/batch/dedup", post(submit_batch_dedup))
        .route("/reports", get(get_reports));

    Router::new()
        .route("/healthz", get(healthz))
        .nest("/api/v1/abis", abis_routes)
        .with_state(state)
}
