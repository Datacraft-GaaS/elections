//! Reports route handler.

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};

use crate::models::AbisReport;
use crate::state::AppState;

pub async fn get_reports(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<AbisReport>) {
    (
        StatusCode::OK,
        Json(AbisReport {
            generated_at: chrono::Utc::now().to_rfc3339(),
            gallery_size: 0,
            pending_dedup_count: 0,
            open_adjudication_count: 0,
            match_throughput_per_hour: 0,
            duplicate_rate_pct: 0.0,
            p99_match_ms: 0,
        }),
    )
}
