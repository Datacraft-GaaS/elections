//! Batch dedup route handler.

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};

use crate::models::{BatchDedupRequest, BatchJobAccepted};
use crate::state::AppState;

pub async fn submit_batch_dedup(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<BatchDedupRequest>,
) -> (StatusCode, Json<BatchJobAccepted>) {
    (
        StatusCode::ACCEPTED,
        Json(BatchJobAccepted {
            job_id: uuid::Uuid::new_v4().to_string(),
            status: "queued".to_string(),
            estimated_duration_s: 3600,
        }),
    )
}
