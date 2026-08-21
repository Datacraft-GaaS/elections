use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use tracing::info;

use crate::models::*;

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
