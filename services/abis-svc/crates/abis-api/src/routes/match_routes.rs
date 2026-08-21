//! Match route handlers: 1:N, 1:1, and result retrieval.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use abis_matcher::{MatchPipeline, MockMatcher};

use crate::models::{Match11Request, Match11Result, Match1NRequest, MatchAccepted, Match1NResult, MatchCandidate};
use crate::state::AppState;

pub async fn match_1n(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<Match1NRequest>,
) -> (StatusCode, Json<MatchAccepted>) {
    let _matcher = MockMatcher;
    let request_id = uuid::Uuid::new_v4().to_string();
    (
        StatusCode::ACCEPTED,
        Json(MatchAccepted {
            request_id,
            status: "queued".to_string(),
            estimated_completion_ms: 500,
        }),
    )
}

pub async fn match_11(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<Match11Request>,
) -> (StatusCode, Json<Match11Result>) {
    let matcher = MockMatcher;
    let result = matcher.match_11(&req.probe_ref, &req.gallery_ref, 0.8);
    let decision = format!("{:?}", result.outcome).to_lowercase();
    (
        StatusCode::OK,
        Json(Match11Result {
            request_id: uuid::Uuid::new_v4().to_string(),
            decision,
            score: result.confidence_score,
            threshold_used: 0.8,
            processed_at: chrono::Utc::now().to_rfc3339(),
        }),
    )
}

pub async fn get_match_result(
    State(_state): State<Arc<AppState>>,
    Path(request_id): Path<String>,
) -> (StatusCode, Json<Match1NResult>) {
    (
        StatusCode::OK,
        Json(Match1NResult {
            request_id,
            status: "complete".to_string(),
            candidates: vec![MatchCandidate {
                gallery_ref: "gallery-001".to_string(),
                score: 0.95,
                decision: "match".to_string(),
            }],
        }),
    )
}
