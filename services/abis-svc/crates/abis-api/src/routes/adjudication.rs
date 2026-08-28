//! Adjudication route handlers.

use std::sync::Arc;

use axum::{
	extract::{Path, Query, State},
	http::StatusCode,
	Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::{
	AdjudicationCase, AdjudicationCasePage, AdjudicationResolutionRequest, AdjudicationStatus,
	ListAdjudicationQuery,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Extended models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssignRequest {
	pub adjudicator_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DecideRequest {
	pub outcome: String,
	pub adjudicator_id: String,
	pub second_adjudicator_id: Option<String>, // required for Tier3 dual sign-off
	pub resolution_notes: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EscalateRequest {
	pub reason: String,
	pub escalated_by: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EvidenceItem {
	pub evidence_id: String,
	pub kind: String,
	pub url: String,
	pub added_at: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /v1/adjudication/cases
#[utoipa::path(
    get,
    path = "/v1/adjudication/cases",
    tag = "adjudication",
    params(
        ("state" = Option<String>, Query, description = "Filter by case state"),
        ("priority" = Option<String>, Query, description = "Filter by priority"),
        ("assigned_to" = Option<String>, Query, description = "Filter by adjudicator"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "List of cases", body = AdjudicationCasePage),
    )
)]
pub async fn list_adjudication(
	State(_state): State<Arc<AppState>>,
	Query(_q): Query<ListAdjudicationQuery>,
) -> (StatusCode, Json<AdjudicationCasePage>) {
	(
		StatusCode::OK,
		Json(AdjudicationCasePage {
			cases: vec![],
			pagination: crate::models::PaginationMeta {
				total: 0,
				page: 1,
				page_size: 50,
				has_next: false,
			},
		}),
	)
}

/// POST /v1/adjudication/cases/{id}/assign
#[utoipa::path(
    post,
    path = "/v1/adjudication/cases/{id}/assign",
    tag = "adjudication",
    params(("id" = String, Path, description = "Case ID")),
    request_body = AssignRequest,
    responses(
        (status = 200, description = "Case assigned"),
    )
)]
pub async fn assign_adjudication(
	State(_state): State<Arc<AppState>>,
	Path(case_id): Path<String>,
	Json(req): Json<AssignRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
	(
		StatusCode::OK,
		Json(serde_json::json!({
			"case_id": case_id,
			"assigned_to": req.adjudicator_id,
			"status": "in_review"
		})),
	)
}

/// POST /v1/adjudication/cases/{id}/decide
#[utoipa::path(
    post,
    path = "/v1/adjudication/cases/{id}/decide",
    tag = "adjudication",
    params(("id" = String, Path, description = "Case ID")),
    request_body = DecideRequest,
    responses(
        (status = 200, description = "Decision recorded"),
        (status = 400, description = "Tier3 requires dual sign-off"),
    )
)]
pub async fn decide_adjudication(
	State(_state): State<Arc<AppState>>,
	Path(case_id): Path<String>,
	Json(req): Json<DecideRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
	// Tier3 dual sign-off check.
	if req.second_adjudicator_id.is_none() && req.adjudicator_id == "tier3" {
		return (
			StatusCode::BAD_REQUEST,
			Json(serde_json::json!({"error": "Tier3 decision requires second_adjudicator_id"})),
		);
	}
	(
		StatusCode::OK,
		Json(serde_json::json!({
			"case_id": case_id,
			"outcome": req.outcome,
			"status": "resolved"
		})),
	)
}

/// POST /v1/adjudication/cases/{id}/escalate
#[utoipa::path(
    post,
    path = "/v1/adjudication/cases/{id}/escalate",
    tag = "adjudication",
    params(("id" = String, Path, description = "Case ID")),
    request_body = EscalateRequest,
    responses(
        (status = 200, description = "Case escalated"),
    )
)]
pub async fn escalate_adjudication(
	State(_state): State<Arc<AppState>>,
	Path(case_id): Path<String>,
	Json(req): Json<EscalateRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
	(
		StatusCode::OK,
		Json(serde_json::json!({
			"case_id": case_id,
			"status": "escalated",
			"reason": req.reason,
		})),
	)
}

/// GET /v1/adjudication/cases/{id}/evidence
#[utoipa::path(
    get,
    path = "/v1/adjudication/cases/{id}/evidence",
    tag = "adjudication",
    params(("id" = String, Path, description = "Case ID")),
    responses(
        (status = 200, description = "Evidence list"),
    )
)]
pub async fn list_evidence(
	State(_state): State<Arc<AppState>>,
	Path(_case_id): Path<String>,
) -> (StatusCode, Json<Vec<EvidenceItem>>) {
	(StatusCode::OK, Json(vec![]))
}

/// POST /v1/adjudication/cases/{id}/evidence
#[utoipa::path(
    post,
    path = "/v1/adjudication/cases/{id}/evidence",
    tag = "adjudication",
    params(("id" = String, Path, description = "Case ID")),
    responses(
        (status = 202, description = "Evidence accepted"),
    )
)]
pub async fn upload_evidence(
	State(_state): State<Arc<AppState>>,
	Path(case_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
	let evidence_id = uuid::Uuid::new_v4().to_string();
	(
		StatusCode::ACCEPTED,
		Json(serde_json::json!({
			"case_id": case_id,
			"evidence_id": evidence_id,
			"status": "accepted",
		})),
	)
}

/// PATCH /v1/adjudication/{case_id} (legacy resolve endpoint, kept for compat)
pub async fn resolve_adjudication(
	State(_state): State<Arc<AppState>>,
	Path(case_id): Path<String>,
	Json(_req): Json<AdjudicationResolutionRequest>,
) -> (StatusCode, Json<AdjudicationCase>) {
	(
		StatusCode::OK,
		Json(AdjudicationCase {
			case_id,
			status: "resolved".to_string(),
			match_score: 0.65,
			probe_ref: "probe-001".into(),
			candidate_gallery_ref: "gallery-001".into(),
			outcome: Some("no_match".into()),
			resolution_notes: None,
			adjudicator_id: None,
			created_at: chrono::Utc::now(),
			resolved_at: Some(chrono::Utc::now()),
		}),
	)
}
