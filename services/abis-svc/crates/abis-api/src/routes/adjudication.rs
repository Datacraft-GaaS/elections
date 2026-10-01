//! Adjudication route handlers.

use std::sync::Arc;

use axum::{
	extract::{Path, Query, State},
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::{AdjudicationResolutionRequest, ListAdjudicationQuery};
use crate::error::ApiError;
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
) -> ApiError {
	ApiError::NotImplemented(
		"adjudication case listing is declared by the abis contract but has no \
		 store behind it; it previously answered 200 with an empty page and \
		 total: 0, which read as a real query that found nothing"
			.to_string(),
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
	Path(_case_id): Path<String>,
	Json(_req): Json<AssignRequest>,
) -> ApiError {
	// Echoed the request back as status "in_review" without storing the
	// assignment, so the case looked assigned to an adjudicator who was never
	// notified and whose queue does not exist.
	 ApiError::NotImplemented(
		"adjudication assignment is declared by the contract but has no store \
		 behind it; it previously echoed the request back as in_review"
			.to_string(),
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
) -> Response {
	// The tier3 dual sign-off check below is real and is kept: a tier3 decision
	// without a second adjudicator is genuinely rejected with 400.
	if req.second_adjudicator_id.is_none() && req.adjudicator_id == "tier3" {
		return ApiError::Validation(
			"Tier3 decision requires second_adjudicator_id".to_string(),
		)
		.into_response();
	}
	// What it used to do next was echo the caller's own outcome back as
	// status "resolved", storing nothing. So the guard decided nothing: the
	// request passed validation and the case was never adjudicated.
	let _ = case_id;
	ApiError::NotImplemented(
		"adjudication decision is declared by the contract but has no store \
		 behind it; it previously echoed the caller's outcome back as resolved"
			.to_string(),
	)
	.into_response()
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
	Path(_case_id): Path<String>,
	Json(_req): Json<EscalateRequest>,
) -> ApiError {
	// Answered 200 with status "escalated" and the reason echoed back. Nothing
	// was escalated: an escalation that reaches no adjudicator reads, from the
	// response, exactly like one that did.
	ApiError::NotImplemented(
		"adjudication escalation is declared by the contract but has no store \
		 behind it; it previously answered 200 with status escalated"
			.to_string(),
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
) -> ApiError {
	// An empty list reads as "this case has no evidence", which is a claim
	// about the case rather than about storage.
	ApiError::NotImplemented(
		"evidence listing is declared by the contract but has no store behind \
		 it; it previously returned an empty list"
			.to_string(),
	)
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
	Path(_case_id): Path<String>,
) -> ApiError {
	// Accepted no upload, stored nothing, and minted an evidence_id so the
	// caller held a receipt for an artefact that was never received.
	ApiError::NotImplemented(
		"evidence upload is declared by the contract but has no store behind \
		 it; it previously returned 202 with a minted evidence_id for an upload \
		 it never received"
			.to_string(),
	)
}

/// PATCH /v1/adjudication/{case_id} (legacy resolve endpoint, kept for compat)
pub async fn resolve_adjudication(
	State(_state): State<Arc<AppState>>,
	Path(_case_id): Path<String>,
	Json(_req): Json<AdjudicationResolutionRequest>,
) -> ApiError {
	// This handler used to answer 200 for every case id with a fabricated
	// verdict: status "resolved", match_score 0.65, outcome "no_match", and
	// probe/gallery refs naming records it never read. On an ABIS endpoint a
	// caller cannot distinguish that from a real biometric comparison, which
	// is a worse failure than refusing: 501 says the answer does not exist.
	ApiError::NotImplemented(
		"adjudication resolution is declared by the abis contract but has no \
		 comparison engine or store behind it; it previously returned a \
		 fabricated match_score and outcome for any case id"
			.to_string(),
	)
}
