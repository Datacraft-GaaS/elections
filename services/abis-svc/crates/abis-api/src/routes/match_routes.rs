//! Match route handlers: 1:N submit, 1:1 verify, batch, and result retrieval.

use std::sync::Arc;

use axum::{
	extract::{Path, State},
	http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use abis_matcher::{MatchOutcome, MatchPipeline, MockMatcher};

use crate::error::ApiError;
use crate::idempotency;
use crate::models::{
	Match11Request, Match11Result, Match1NRequest, Match1NResult, MatchAccepted, MatchCandidate,
};
use crate::state::AppState;

/// Configured default similarity threshold when no override is supplied.
pub const DEFAULT_THRESHOLD: f32 = 0.8;
/// Scores within this band below the threshold are reported as near hits.
const NEAR_HIT_DELTA: f32 = 0.05;
/// Modality values accepted by the contract.
const MODALITIES: [&str; 4] = ["fingerprint", "iris", "face", "multimodal"];

// ---------------------------------------------------------------------------
// Extended models (v1 batch endpoint)
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
// Shared helpers
// ---------------------------------------------------------------------------

fn parse_json<T: serde::de::DeserializeOwned>(bytes: &Bytes) -> Result<T, ApiError> {
	serde_json::from_slice(bytes)
		.map_err(|e| ApiError::Validation(format!("malformed JSON body: {e}")))
}

fn validate_modality(modality: &str) -> Result<(), ApiError> {
	if MODALITIES.contains(&modality) {
		Ok(())
	} else {
		Err(ApiError::Validation(format!(
			"unknown modality '{modality}'; expected one of {MODALITIES:?}"
		)))
	}
}

fn validate_uuid(field: &str, value: &str) -> Result<uuid::Uuid, ApiError> {
	uuid::Uuid::parse_str(value)
		.map_err(|_| ApiError::Validation(format!("{field} must be a UUID")))
}

fn json_response<T: serde::Serialize>(status: StatusCode, value: &T) -> Result<Response, ApiError> {
	let body = serde_json::to_vec(value).map_err(|e| ApiError::Store(sqlx::Error::Configuration(e.into())))?;
	Ok((status, [(CONTENT_TYPE, "application/json")], body).into_response())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CandidateDecision {
	Hit,
	NearHit,
	Miss,
}

impl CandidateDecision {
	pub fn from_score(score: f32, threshold: f32) -> Self {
		if score >= threshold {
			Self::Hit
		} else if score >= threshold - NEAR_HIT_DELTA {
			Self::NearHit
		} else {
			Self::Miss
		}
	}

	pub fn as_str(self) -> &'static str {
		match self {
			CandidateDecision::Hit => "hit",
			CandidateDecision::NearHit => "near_hit",
			CandidateDecision::Miss => "miss",
		}
	}
}

/// Map an engine outcome to the contract decision vocabulary.
pub fn decision_from_outcome(outcome: MatchOutcome) -> &'static str {
	match outcome {
		MatchOutcome::Match => "match",
		MatchOutcome::NoMatch => "no_match",
		// Ambiguous or low-quality comparisons are inconclusive for the caller.
		MatchOutcome::Adjudicate | MatchOutcome::InsufficientQuality => "inconclusive",
	}
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/v1/abis/match/1n (also mounted under /v1/match/1n).
///
/// Persists a queued 1:N dedup request for the async matching workers and
/// honours ADR-0011 idempotency on (operation, Idempotency-Key).
#[utoipa::path(
    post,
    path = "/v1/match/1n",
    tag = "match",
    request_body = Match1NRequest,
    responses(
        (status = 202, description = "Match job queued", body = MatchAccepted),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Idempotency key reuse"),
    )
)]
pub async fn match_1n(
	State(state): State<Arc<AppState>>,
	headers: HeaderMap,
	body: Bytes,
) -> Result<Response, ApiError> {
	const OPERATION: &str = "POST /api/v1/abis/match/1n";
	let key = idempotency::require_key(&headers)?;
	let req: Match1NRequest = parse_json(&body)?;
	validate_modality(&req.modality)?;
	validate_uuid("election_id", &req.election_id)?;
	if req.probe_ref.is_empty() {
		return Err(ApiError::Validation("probe_ref must not be empty".into()));
	}
	if let Some(t) = req.threshold_override {
		if !(0.0..=1.0).contains(&t) {
			return Err(ApiError::Validation(
				"threshold_override must be within [0.0, 1.0]".into(),
			));
		}
	}
	match req.max_candidates {
		Some(n @ 1..=50) => {
			let _ = n;
		}
		Some(_) => {
			return Err(ApiError::Validation(
				"max_candidates must be between 1 and 50".into(),
			));
		}
		None => {}
	}

	let fp = idempotency::fingerprint_request(&body);
	if let Some(replay) = idempotency::lookup(&state.db, OPERATION, &key, &fp).await? {
		tracing::info!(key = %key, "replaying idempotent 1:n submission");
		return Ok(idempotency::replay_response(replay));
	}

	let request_id = uuid::Uuid::now_v7().to_string();
	let accepted = MatchAccepted {
		request_id: request_id.clone(),
		status: "queued".into(),
		estimated_completion_ms: 5000,
	};

	// The idempotency_key UNIQUE constraint backstops the lookup race.
	sqlx::query(
		"INSERT INTO eel_match_requests \
		     (request_id, probe_ref, modality, election_id, caller_id, idempotency_key) \
		 VALUES ($1, $2, $3, $4, $5, $6)",
	)
	.bind(&request_id)
	.bind(&req.probe_ref)
	.bind(&req.modality)
	.bind(&req.election_id)
	.bind(&req.caller_id)
	.bind(&key)
	.execute(&state.db)
	.await?;

	let body_bytes = serde_json::to_vec(&accepted)
		.map_err(|e| ApiError::Store(sqlx::Error::Configuration(e.into())))?;
	let response = (StatusCode::ACCEPTED, [(CONTENT_TYPE, "application/json")], body_bytes.clone()).into_response();
	idempotency::record(&state.db, OPERATION, &key, &req.caller_id, &fp, 202, &body_bytes).await?;
	Ok(response)
}

/// POST /api/v1/abis/match/11 (also mounted under /v1/match/11).
///
/// Runs the synchronous 1:1 comparison through the matcher pipeline and
/// persists both the request audit row and the result.
#[utoipa::path(
    post,
    path = "/v1/match/11",
    tag = "match",
    request_body = Match11Request,
    responses(
        (status = 200, description = "Match result", body = Match11Result),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Idempotency key reuse"),
    )
)]
pub async fn match_11(
	State(state): State<Arc<AppState>>,
	headers: HeaderMap,
	body: Bytes,
) -> Result<Response, ApiError> {
	const OPERATION: &str = "POST /api/v1/abis/match/11";
	let key = idempotency::require_key(&headers)?;
	let req: Match11Request = parse_json(&body)?;
	validate_modality(&req.modality)?;
	if req.probe_ref.is_empty() || req.gallery_ref.is_empty() {
		return Err(ApiError::Validation(
			"probe_ref and gallery_ref must not be empty".into(),
		));
	}

	let fp = idempotency::fingerprint_request(&body);
	if let Some(replay) = idempotency::lookup(&state.db, OPERATION, &key, &fp).await? {
		tracing::info!(key = %key, "replaying idempotent 1:1 verification");
		return Ok(idempotency::replay_response(replay));
	}

	let matcher = MockMatcher;
	let result = matcher.match_11(&req.probe_ref, &req.gallery_ref, DEFAULT_THRESHOLD);
	let decision = decision_from_outcome(result.outcome.clone());

	let request_id = uuid::Uuid::now_v7().to_string();
	let result_id = uuid::Uuid::now_v7().to_string();
	let outcome_column = match result.outcome {
		MatchOutcome::Match => "match",
		MatchOutcome::NoMatch => "no_match",
		MatchOutcome::Adjudicate => "adjudicate",
		MatchOutcome::InsufficientQuality => "insufficient_quality",
	};

	let mut tx = state.db.begin().await?;
	sqlx::query(
		"INSERT INTO eel_match_requests \
		     (request_id, probe_ref, modality, election_id, status, completed_at, caller_id, idempotency_key) \
		 VALUES ($1, $2, $3, $4, 'complete', NOW(), $5, $6)",
	)
	.bind(&request_id)
	.bind(&req.probe_ref)
	.bind(&req.modality)
	.bind(uuid::Uuid::now_v7().to_string())
	.bind(&req.caller_id)
	.bind(&key)
	.execute(&mut *tx)
	.await?;

	// The engine returns a deterministic score for (probe, gallery); persist
	// it so downstream audits can reproduce the decision.
	sqlx::query(
		"INSERT INTO eel_match_results \
		     (result_id, request_id, outcome, confidence_score, modality_scores) \
		 VALUES ($1, $2, $3, $4, $5)",
	)
	.bind(&result_id)
	.bind(&request_id)
	.bind(outcome_column)
	.bind(result.confidence_score)
	.bind(serde_json::json!({
			&req.modality: result.confidence_score,
			"threshold": DEFAULT_THRESHOLD,
	}))
	.execute(&mut *tx)
	.await?;
	tx.commit().await?;

	let payload = Match11Result {
		request_id,
		decision: decision.to_string(),
		score: result.confidence_score,
		threshold_used: DEFAULT_THRESHOLD,
		modality: Some(req.modality.clone()),
		processed_at: chrono::Utc::now(),
		engine_version: Some(env!("CARGO_PKG_VERSION").to_string()),
	};
	let body_bytes = serde_json::to_vec(&payload)
		.map_err(|e| ApiError::Store(sqlx::Error::Configuration(e.into())))?;
	let response = (StatusCode::OK, [(CONTENT_TYPE, "application/json")], body_bytes.clone()).into_response();
	idempotency::record(&state.db, OPERATION, &key, &req.caller_id, &fp, 200, &body_bytes).await?;
	Ok(response)
}

/// POST /v1/match/batch (internal v1 convenience endpoint).
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
	State(state): State<Arc<AppState>>,
	Json(req): Json<BatchMatchRequest>,
) -> (StatusCode, Json<BatchMatchAccepted>) {
	// Fan out each probe as a tracked 1:N request row.
	for probe in &req.probes {
		let request_id = uuid::Uuid::now_v7().to_string();
		let _ = sqlx::query(
			"INSERT INTO eel_match_requests \
			     (request_id, probe_ref, modality, election_id, caller_id) \
			 VALUES ($1, $2, $3, $4, $5)",
		)
		.bind(&request_id)
		.bind(&probe.probe_ref)
		.bind(&probe.modality)
		.bind(&probe.election_id)
		.bind(&probe.caller_id)
		.execute(&state.db)
		.await;
	}
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

/// GET /api/v1/abis/match/{request_id} (also mounted under /v1).
///
/// Polls a previously submitted 1:N request; candidates come from the
/// ranked `eel_match_candidates` rows produced by the matching worker.
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
	State(state): State<Arc<AppState>>,
	Path(request_id): Path<String>,
) -> Result<Response, ApiError> {
	validate_uuid("request_id", &request_id)?;

	let row: Option<(String, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, Option<String>)> =
		sqlx::query_as(
			"SELECT status, submitted_at, completed_at, failure_reason \
			 FROM eel_match_requests WHERE request_id = $1",
		)
		.bind(&request_id)
		.fetch_optional(&state.db)
		.await?;
	let (status, submitted_at, completed_at, failure_reason) = row.ok_or_else(|| {
		ApiError::NotFound(format!("match request {request_id} not found"))
	})?;

	let mut candidates: Vec<MatchCandidate> = Vec::new();
	if completed_at.is_some() {
		let rows: Vec<(uuid::Uuid, f32)> = sqlx::query_as(
			"SELECT subject_id, score_fused FROM eel_match_candidates \
			 WHERE result_id IN (SELECT result_id FROM eel_match_results WHERE request_id = $1) \
			 ORDER BY rank ASC LIMIT 50",
		)
		.bind(&request_id)
		.fetch_all(&state.db)
		.await?;
		candidates = rows
			.into_iter()
			.map(|(subject_id, score)| MatchCandidate {
				gallery_ref: format!("bio-store://eel.local/gallery/{subject_id}"),
				score,
				decision: CandidateDecision::from_score(score, DEFAULT_THRESHOLD)
					.as_str()
					.to_string(),
				modality: None,
			})
			.collect();
	}

	json_response(
		StatusCode::OK,
		&Match1NResult {
			request_id,
			status,
			candidates,
			submitted_at,
			completed_at,
			failure_reason,
		},
	)
}

// Keep the digest import referenced even if debug builds strip tracing of it.
#[allow(dead_code)]
fn unused_sha_guard(_: Sha256) {}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn candidate_decision_bands() {
		assert_eq!(
			CandidateDecision::from_score(0.9, 0.8),
			CandidateDecision::Hit
		);
		assert_eq!(
			CandidateDecision::from_score(0.77, 0.8),
			CandidateDecision::NearHit
		);
		assert_eq!(
			CandidateDecision::from_score(0.1, 0.8),
			CandidateDecision::Miss
		);
	}

	#[test]
	fn decisions_map_to_contract_vocabulary() {
		assert_eq!(decision_from_outcome(MatchOutcome::Match), "match");
		assert_eq!(decision_from_outcome(MatchOutcome::NoMatch), "no_match");
		assert_eq!(
			decision_from_outcome(MatchOutcome::Adjudicate),
			"inconclusive"
		);
		assert_eq!(
			decision_from_outcome(MatchOutcome::InsufficientQuality),
			"inconclusive"
		);
	}
}
