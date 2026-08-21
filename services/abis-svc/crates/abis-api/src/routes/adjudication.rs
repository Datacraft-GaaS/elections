//! Adjudication route handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use crate::models::{
    AdjudicationCase, AdjudicationCasePage, AdjudicationResolutionRequest, AdjudicationStatus,
    ListAdjudicationQuery,
};
use crate::state::AppState;

pub async fn list_adjudication(
    State(_state): State<Arc<AppState>>,
    Query(_q): Query<ListAdjudicationQuery>,
) -> (StatusCode, Json<AdjudicationCasePage>) {
    (
        StatusCode::OK,
        Json(AdjudicationCasePage {
            cases: vec![],
            total: 0,
        }),
    )
}

pub async fn resolve_adjudication(
    State(_state): State<Arc<AppState>>,
    Path(case_id): Path<String>,
    Json(_req): Json<AdjudicationResolutionRequest>,
) -> (StatusCode, Json<AdjudicationCase>) {
    (
        StatusCode::OK,
        Json(AdjudicationCase {
            case_id,
            status: AdjudicationStatus::Resolved,
            match_score: 0.65,
            probe_ref: "probe-001".to_string(),
            candidate_gallery_ref: "gallery-001".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }),
    )
}
