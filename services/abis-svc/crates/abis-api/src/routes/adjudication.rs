// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Adjudication route handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::models::{AdjudicationResolutionRequest, ListAdjudicationQuery};
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

/// One piece of evidence attached to a case.
///
/// `evidence_id` and `added_at` are assigned by the service: a caller that had to
/// supply them would be handing back its own receipt, which is the fabrication this
/// endpoint used to perform by minting an id for an upload it never received.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EvidenceItem {
    pub evidence_id: String,
    pub kind: String,
    pub url: String,
    pub added_at: String,
}

/// The body of an evidence upload: what the artefact is and where it lives.
///
/// The artefact itself is in object storage, so `url` is a reference and not a
/// payload.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UploadEvidenceRequest {
    pub kind: String,
    pub url: String,
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
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListAdjudicationQuery>,
) -> Response {
    // Previously answered 200 with an empty page and `total: 0`, which read as a
    // real query that found nothing rather than as a service holding no cases.
    //
    // `min_score` is a real filter and not an ignored parameter: a case that has not
    // been decided has no comparison behind it, so it has no score and cannot
    // satisfy a minimum. That is why an undecided case is not returned by
    // `min_score=0`.
    let cases = state
        .adjudication
        .cases_matching(q.status.as_deref(), q.min_score.map(f64::from));
    let page_size = q.page_size.unwrap_or(50).max(1) as usize;
    let page_no = q.page.unwrap_or(1).max(1) as usize;
    let total = cases.len();
    let start = (page_no - 1) * page_size;
    let items: Vec<&crate::adjudication::Case> = cases.iter().skip(start).take(page_size).collect();
    (
        axum::http::StatusCode::OK,
        Json(serde_json::json!({
            "items": items,
            "total": total,
            "page": page_no,
            "page_size": page_size,
        })),
    )
        .into_response()
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
    State(state): State<Arc<AppState>>,
    Path(case_id): Path<String>,
    Json(req): Json<AssignRequest>,
) -> Response {
    // Previously echoed the request back as `status: "in_review"` without storing
    // the assignment, so the case looked assigned to an adjudicator who was never
    // notified and whose queue did not exist.
    if req.adjudicator_id.trim().is_empty() {
        return ApiError::Validation("adjudicator_id is required".to_string()).into_response();
    }
    match state
        .adjudication
        .assign(&case_id, req.adjudicator_id.trim())
    {
        Ok(case) => (axum::http::StatusCode::OK, Json(case)).into_response(),
        Err(e) if e.starts_with("no case") => ApiError::NotFound(e).into_response(),
        Err(e) => ApiError::Conflict(e).into_response(),
    }
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
    State(state): State<Arc<AppState>>,
    Path(case_id): Path<String>,
    Json(req): Json<DecideRequest>,
) -> Response {
    // The tier3 dual sign-off check is real and is kept: a tier3 decision without a
    // second adjudicator is genuinely rejected.
    if req.second_adjudicator_id.is_none() && req.adjudicator_id == "tier3" {
        return ApiError::Validation("Tier3 decision requires second_adjudicator_id".to_string())
            .into_response();
    }
    // What it used to do next was echo the caller's own outcome back as
    // `status: "resolved"`, storing nothing. So the guard decided nothing: the
    // request passed validation and the case was never adjudicated.
    //
    // `match_score` is not accepted from the caller. This endpoint decides a case; it
    // does not compare templates, so a caller-supplied score would be the same
    // fabrication as the old hardcoded 0.65, just arriving over the wire.
    match state.adjudication.decide(
        &case_id,
        req.outcome.trim(),
        req.adjudicator_id.trim(),
        req.second_adjudicator_id.as_deref().map(str::trim),
        req.resolution_notes.trim(),
        None,
    ) {
        Ok(case) => (axum::http::StatusCode::OK, Json(case)).into_response(),
        Err(e) if e.starts_with("no case") => ApiError::NotFound(e).into_response(),
        Err(e) => ApiError::Conflict(e).into_response(),
    }
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
    State(state): State<Arc<AppState>>,
    Path(case_id): Path<String>,
    Json(req): Json<EscalateRequest>,
) -> Response {
    // Previously answered 200 with `status: "escalated"` and the reason echoed back.
    // Nothing was escalated, and an escalation that reaches no adjudicator reads,
    // from the response, exactly like one that did.
    let reason = req.reason.trim();
    if reason.is_empty() || req.escalated_by.trim().is_empty() {
        return ApiError::Validation(
            "reason and escalated_by are required: an escalation names who sent it and \
			 why, or it is an anonymous escalation that no queue can triage"
                .to_string(),
        )
        .into_response();
    }
    match state.adjudication.escalate(&case_id, reason) {
        Ok(case) => (axum::http::StatusCode::OK, Json(case)).into_response(),
        Err(e) if e.starts_with("no case") => ApiError::NotFound(e).into_response(),
        Err(e) => ApiError::Conflict(e).into_response(),
    }
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
    State(state): State<Arc<AppState>>,
    Path(case_id): Path<String>,
) -> Response {
    // An empty list reads as "this case has no evidence", which is a claim about
    // the case rather than about storage. An unknown case must be distinguishable
    // from a case with no evidence, so this 404s rather than returning `[]`.
    match state.adjudication.case(&case_id) {
        Some(case) => Json(serde_json::json!({ "evidence": case.evidence })).into_response(),
        None => ApiError::NotFound(format!("no case {case_id}")).into_response(),
    }
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
    State(state): State<Arc<AppState>>,
    Path(case_id): Path<String>,
    Json(req): Json<UploadEvidenceRequest>,
) -> Response {
    // Previously accepted no upload, stored nothing, and minted an evidence_id, so
    // the caller held a receipt for an artefact that was never received.
    //
    // This records that evidence exists and where it lives. The artefact itself is
    // in object storage, so `url` is a reference and not a payload -- and an empty
    // one is refused rather than recorded.
    let kind = req.kind.trim();
    let url = req.url.trim();
    if kind.is_empty() || url.is_empty() {
        return ApiError::Validation(
            "kind and url are required: this endpoint records that evidence exists and \
			 where it is, so an entry with neither is a receipt for nothing"
                .to_string(),
        )
        .into_response();
    }
    match state.adjudication.attach_evidence(&case_id, kind, url) {
        Ok(evidence) => (
            axum::http::StatusCode::CREATED,
            Json(serde_json::json!({
                "case_id": case_id,
                "evidence_id": evidence.evidence_id,
                "kind": evidence.kind,
                "url": evidence.url,
                "added_at": evidence.added_at,
            })),
        )
            .into_response(),
        Err(e) if e.starts_with("no case") => ApiError::NotFound(e).into_response(),
        Err(e) => ApiError::Conflict(e).into_response(),
    }
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
    //
    // The earlier version of this comment said "no comparison engine ... behind it".
    // That was wrong, and it was the kind of wrong that costs the next reader an
    // afternoon: `abis-matcher::adjudication` provides `open_adjudication_case` and
    // `record_decision`, and `abis-adjudication` is a whole crate. What is missing
    // is a **MatchResult over real templates** -- the input `open_adjudication_case`
    // takes -- and there is no template store or capture corpus behind this service.
    // The only matcher in the workspace is `abis_matcher::mock_matcher::MockMatcher`,
    // and wiring that into a citizen-facing endpoint would be the same fabrication
    // with a different number.
    ApiError::NotImplemented(
        "adjudication resolution needs a match result over real biometric templates, \
		 and no template store or capture corpus is reachable from this service. The \
		 adjudication engine itself is present (abis-matcher::adjudication, and the \
		 abis-adjudication crate); what is absent is the MatchResult it consumes. The \
		 only matcher in the workspace is MockMatcher, which must not answer this \
		 endpoint. This previously returned a fabricated match_score and outcome for \
		 any case id"
            .to_string(),
    )
}
