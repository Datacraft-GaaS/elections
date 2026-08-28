use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::models::*;
use crate::spec;
use crate::state::SharedState;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn submit_finding(Json(req): Json<SubmitFindingRequest>) -> impl IntoResponse {
    info!(
        election_cycle = %req.election_cycle,
        finding_type = ?req.finding_type,
        severity = ?req.severity,
        "submit_finding"
    );
    let finding = Finding {
        id: uuid::Uuid::new_v4().to_string(),
        election_cycle: req.election_cycle,
        submitter_category: req.submitter_category,
        submitter_id: req.submitter_id,
        finding_type: req.finding_type,
        severity: req.severity,
        description: req.description,
        evidence_refs: req.evidence_refs,
        recommended_action: req.recommended_action,
        is_long_drip: false,
        duplicate_of: None,
        status: FindingStatus::Submitted,
        submitted_at: chrono::Utc::now().to_rfc3339(),
    };
    (StatusCode::CREATED, Json(finding))
}

pub async fn list_findings() -> impl IntoResponse {
    info!("list_findings");
    let findings: Vec<Finding> = vec![];
    Json(serde_json::json!({ "findings": findings }))
}

pub async fn triage_finding(
    Path(id): Path<String>,
    Json(req): Json<TriageFindingRequest>,
) -> impl IntoResponse {
    info!(id, status = ?req.status, "triage_finding");
    StatusCode::NO_CONTENT
}

pub async fn list_recommendations() -> impl IntoResponse {
    info!("list_recommendations");
    let recs: Vec<Recommendation> = vec![];
    Json(serde_json::json!({ "recommendations": recs }))
}

pub async fn sign_recommendation(
    Path(id): Path<String>,
    Json(req): Json<SignRecommendationRequest>,
) -> impl IntoResponse {
    info!(id, signed_by = %req.signed_by, "sign_recommendation");
    StatusCode::NO_CONTENT
}

pub async fn public_tracker() -> impl IntoResponse {
    info!("public_tracker");
    let items: Vec<PublicTrackerItem> = vec![];
    Json(serde_json::json!({ "items": items }))
}

pub async fn create_parliament_referral(
    Json(req): Json<CreateParliamentReferralRequest>,
) -> impl IntoResponse {
    info!(
        recommendation_count = req.recommendation_ids.len(),
        "create_parliament_referral"
    );
    let referral = ParliamentReferral {
        id: uuid::Uuid::new_v4().to_string(),
        recommendation_ids: req.recommendation_ids,
        referral_package_hash: req.referral_package_hash,
        ..Default::default()
    };
    (StatusCode::CREATED, Json(referral))
}

pub async fn record_parliament_response(
    Path(id): Path<String>,
    Json(req): Json<RecordParliamentResponseRequest>,
) -> impl IntoResponse {
    info!(
        id,
        parliament_response = %req.parliament_response,
        "record_parliament_response"
    );
    StatusCode::NO_CONTENT
}

pub async fn list_long_drip() -> impl IntoResponse {
    info!("list_long_drip");
    let alerts: Vec<LongDripAlert> = vec![];
    Json(serde_json::json!({ "alerts": alerts }))
}

pub async fn record_long_drip_review(
    Path(id): Path<String>,
    Json(req): Json<RecordLongDripReviewRequest>,
) -> impl IntoResponse {
    info!(
        id,
        reviewer_id = %req.reviewer_id,
        escalated = req.escalated_to_commissioner,
        "record_long_drip_review"
    );
    StatusCode::NO_CONTENT
}

pub async fn next_cycle_handoff() -> impl IntoResponse {
    info!("next_cycle_handoff");
    Json(serde_json::json!({ "handoff_items": [] }))
}

// ---------------------------------------------------------------------------
// Public contract endpoints (governance-review/post-election.yaml)
// ---------------------------------------------------------------------------

/// POST /api/v1/governance/review/reviews — initiate a post-election review.
pub async fn initiate_post_election_review(
    State(state): State<SharedState>,
    Json(req): Json<spec::ReviewInitRequest>,
) -> impl IntoResponse {
    info!(election_id = %req.election_id, "initiate_post_election_review");
    if req.reviewers.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "at least one reviewer is required"})),
        )
            .into_response();
    }
    let now = chrono::Utc::now().to_rfc3339();
    let review = spec::ElectionReview {
        review_id: uuid::Uuid::now_v7().to_string(),
        election_id: req.election_id,
        scope: req.scope,
        scope_code: req.scope_code,
        reviewers: req.reviewers,
        status: "initiated".to_string(),
        report_url: None,
        created_at: now.clone(),
        updated_at: now,
    };
    let mut guard = state.reviews.lock().expect("lock poisoned");
    guard.insert(review.review_id.clone(), review.clone());
    drop(guard);
    (StatusCode::CREATED, Json(review)).into_response()
}

/// GET /api/v1/governance/review/reviews — list post-election reviews.
pub async fn list_post_election_reviews(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_post_election_reviews");
    let guard = state.reviews.lock().expect("lock poisoned");
    let mut items: Vec<spec::ElectionReview> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }))
}

/// GET /api/v1/governance/review/reviews/{review_id} — fetch one review.
pub async fn get_post_election_review(
    State(state): State<SharedState>,
    Path(review_id): Path<String>,
) -> impl IntoResponse {
    info!(review_id, "get_post_election_review");
    let guard = state.reviews.lock().expect("lock poisoned");
    match guard.get(&review_id) {
        Some(r) => Json(r.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "review not found"})),
        )
            .into_response(),
    }
}

/// POST /api/v1/governance/review/reviews/{review_id}/findings — add a finding.
pub async fn add_review_finding(
    State(state): State<SharedState>,
    Path(review_id): Path<String>,
    Json(req): Json<spec::ReviewFindingRequest>,
) -> impl IntoResponse {
    info!(review_id, "add_review_finding");
    if !state.reviews.lock().expect("lock poisoned").contains_key(&review_id) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "review not found"})),
        )
            .into_response();
    }
    let finding = spec::ReviewFinding {
        finding_id: uuid::Uuid::now_v7().to_string(),
        review_id: review_id.clone(),
        category: req.category,
        severity: req.severity,
        description: req.description,
        recommendation: req.recommendation,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut guard = state.findings.lock().expect("lock poisoned");
    guard.entry(review_id).or_default().push(finding.clone());
    drop(guard);
    {
        let mut rguard = state.reviews.lock().expect("lock poisoned");
        if let Some(r) = rguard.get_mut(&finding.review_id) {
            r.status = "analysing".to_string();
            r.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }
    (StatusCode::CREATED, Json(finding)).into_response()
}

/// POST /api/v1/governance/review/reviews/{review_id}/publish — publish the report.
pub async fn publish_review_report(
    State(state): State<SharedState>,
    Path(review_id): Path<String>,
) -> impl IntoResponse {
    info!(review_id, "publish_review_report");
    let mut guard = state.reviews.lock().expect("lock poisoned");
    match guard.get_mut(&review_id) {
        Some(r) => {
            r.status = "published".to_string();
            r.report_url = Some(format!("https://reports.eel.local/review/{review_id}"));
            r.updated_at = chrono::Utc::now().to_rfc3339();
            Json(r.clone()).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "review not found"})),
        )
            .into_response(),
    }
}
