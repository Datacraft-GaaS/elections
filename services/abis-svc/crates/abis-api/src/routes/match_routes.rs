//! Match route handlers: 1:N, 1:1, batch, and result retrieval.

use std::sync::Arc;

use axum::{
	extract::{Path, State},
	http::StatusCode,
	Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use abis_matcher::{MatchPipeline, MockMatcher};

use crate::models::{
	Match11Request, Match11Result, Match1NRequest, Match1NResult, MatchAccepted, MatchCandidate,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Extended models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchMatchRequest {
	pub probes: Vec<Match1NRequest>,
	pub max_candidates: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchMatchAccepted {
	pub batch_id: String,
	pub probe_count: usize,
	pub status: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /v1/match/1n
#[utoipa::path(
    post,
    path = "/v1/match/1n",
    tag = "match",
    request_body = Match1NRequest,
    responses(
        (status = 202, description = "Match job queued", body = MatchAccepted),
    )
)]
pub async fn match_1n(
	State(_state): State<Arc<AppState>>,
	Json(_req): Json<Match1NRequest>,
) -> (StatusCode, Json<MatchAccepted>) {
	let request_id = uuid::Uuid::new_v4().to_string();
	(
		StatusCode::ACCEPTED,
		Json(MatchAccepted {
			request_id,
			status: "queued".into(),
			estimated_completion_ms: 500,
		}),
	)
}

/// POST /v1/match/11
#[utoipa::path(
    post,
    path = "/v1/match/11",
    tag = "match",
    request_body = Match11Request,
    responses(
        (status = 200, description = "Match result", body = Match11Result),
    )
)]
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

/// POST /v1/match/batch
#[utoipa::path(
    post,
    path = "/v1/match/batch",
    tag = "match",
    request_body = BatchMatchRequest,
    responses(
        (status = 202, description = "Batch accepted", body = BatchMatchAccepted),
    )
)]
pub async fn match_batch(
	State(_state): State<Arc<AppState>>,
	Json(req): Json<BatchMatchRequest>,
) -> (StatusCode, Json<BatchMatchAccepted>) {
	let batch_id = uuid::Uuid::new_v4().to_string();
	(
		StatusCode::ACCEPTED,
		Json(BatchMatchAccepted {
			batch_id,
			probe_count: req.probes.len(),
			status: "queued".into(),
		}),
	)
}

/// GET /v1/match/{request_id}
#[utoipa::path(
    get,
    path = "/v1/match/{request_id}",
    tag = "match",
    params(("request_id" = String, Path, description = "Match request ID")),
    responses(
        (status = 200, description = "Match result", body = Match1NResult),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_match_result(
	State(_state): State<Arc<AppState>>,
	Path(request_id): Path<String>,
) -> (StatusCode, Json<Match1NResult>) {
	(
		StatusCode::OK,
		Json(Match1NResult {
			request_id,
			status: "complete".into(),
			candidates: vec![MatchCandidate {
				gallery_ref: "gallery-001".into(),
				score: 0.95,
				decision: "match".into(),
			}],
		}),
	)
}
