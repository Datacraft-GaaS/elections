use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tracing::info;

use crate::models::*;
use crate::spec;
use crate::state::SharedState;

/// A problem+json response, so a refusal is readable rather than a bare status.
fn problem(status: StatusCode, detail: String) -> Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail,
        })),
    )
        .into_response()
}


pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn submit_finding(State(state): State<SharedState>, Json(req): Json<SubmitFindingRequest>) -> impl IntoResponse {
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
    // Real. Previously built the finding, answered 201 and dropped it, so a
    // finding submitted to the post-election review could never be listed or
    // triaged.
    state
        .findings_by_id
        .insert(finding.id.clone(), finding.clone());
    (StatusCode::CREATED, Json(finding))
}

pub async fn list_findings(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_findings");
    // Real. Previously always an empty array.
    Json(serde_json::json!({
        "findings": state.findings_by_id.list(),
    })).into_response()
}

pub async fn triage_finding(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<TriageFindingRequest>,
) -> Result<Response, Response> {
    info!(id, status = ?req.status, "triage_finding");
    // Real. Previously returned 204 for any id, so a finding could be reported as
    // triaged when it had never been submitted.
    let long_drip = req.is_long_drip;
    let dup = req.duplicate_of.clone();
    let updated = state.findings_by_id.update(&id, |mut f| {
        f.status = req.status;
        if let Some(d) = dup.clone() {
            f.duplicate_of = Some(d);
        }
        if let Some(ld) = long_drip {
            f.is_long_drip = ld;
        }
        f
    });
    match updated {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no finding with id {id}"),
        )),
    }
}

pub async fn list_recommendations(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_recommendations");
    // Real. Previously always an empty array.
    Json(serde_json::json!({
        "recommendations": state.recommendations.list(),
    })).into_response()
}

pub async fn sign_recommendation(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<SignRecommendationRequest>,
) -> Result<Response, Response> {
    info!(id, signed_by = %req.signed_by, "sign_recommendation");
    if req.hsm_signature.trim().is_empty() {
        // The request carries an HSM signature and this handler previously
        // discarded it entirely, logging only the signer's name and answering
        // 204. A recommendation marked signed with no signature recorded cannot
        // be audited, so an absent signature is refused rather than accepted.
        return Err(problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "hsm_signature is required: a recommendation must not be marked \
             signed without the signature that signed it"
                .to_string(),
        ));
    }
    let signature = req.hsm_signature.clone();
    let signed_by = req.signed_by.clone();
    // The Recommendation model has no field for the signature, so it is written
    // to the audit trail rather than discarded. It is recorded verbatim and NOT
    // verified: there is no HSM public key in this workspace to check it against,
    // and dropping it is what let a recommendation read as signed with no
    // signature behind it.
    info!(
        recommendation_id = %id,
        signature_present = !signature.is_empty(),
        signature = %signature,
        "chair signature recorded but NOT verified: no HSM public key here"
    );
    let now = chrono::Utc::now().to_rfc3339();
    // Recorded verbatim. Verification against the HSM public key is NOT
    // implemented here, so this stores the presented signature without checking
    // it -- which is why the field is kept alongside the signer rather than
    // discarded, and why a reviewer can see it has not been verified.
    let updated = state.recommendations.update(&id, |mut r| {
        r.chair_signed_by = Some(signed_by.clone());
        r.chair_signed_at = Some(now.clone());
        r
    });
    match updated {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no recommendation with id {id}"),
        )),
    }
}

pub async fn public_tracker(State(state): State<SharedState>) -> impl IntoResponse {
    info!("public_tracker");
    // Real. Previously always an empty items array, so the public tracker -- the
    // face of the post-election review -- showed nothing at all.
    let mut items: Vec<PublicTrackerItem> = state
        .recommendations
        .list()
        .into_iter()
        .map(|r| PublicTrackerItem {
            recommendation_id: r.id,
            recommendation_text: r.recommendation_text,
            eel_response_type: r.eel_response_type,
            status: if r.chair_signed_at.is_some() {
                "signed".to_string()
            } else {
                "awaiting_chair_signature".to_string()
            },
            chair_signed_at: r.chair_signed_at,
        })
        .collect();
    items.sort_by(|a, b| a.recommendation_id.cmp(&b.recommendation_id));
    Json(serde_json::json!({ "items": items })).into_response()
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
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<RecordParliamentResponseRequest>,
) -> Result<Response, Response> {
    info!(
        id,
        parliament_response = %req.parliament_response,
        "record_parliament_response"
    );
    let response = req.parliament_response.clone();
    let bill = req.bill_reference.clone();
    let at = req.parliament_response_at.clone();
    // Real. Previously returned 204 for any id, discarding the response, the bill
    // reference and the date entirely.
    let updated = state.referrals.update(&id, |mut r| {
        r.parliament_response = Some(response.clone());
        r.bill_reference = bill.clone();
        r.parliament_response_at = Some(at.clone());
        r.clerk_received_at = Some(chrono::Utc::now().to_rfc3339());
        r
    });
    match updated {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no parliament referral with id {id}"),
        )),
    }
}

pub async fn list_long_drip(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_long_drip");
    // Real. Previously always an empty alerts array, which asserted that no
    // long-drip finding was outstanding.
    Json(serde_json::json!({
        "alerts": state.long_drip.list(),
    })).into_response()
}

pub async fn record_long_drip_review(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<RecordLongDripReviewRequest>,
) -> Result<Response, Response> {
    info!(
        id,
        reviewer_id = %req.reviewer_id,
        escalated = req.escalated_to_commissioner,
        "record_long_drip_review"
    );
    let reviewer = req.reviewer_id.clone();
    let escalated = req.escalated_to_commissioner;
    let now = chrono::Utc::now().to_rfc3339();
    // Real. Previously returned 204 for any id, recording no reviewer, no review
    // timestamp and no escalation.
    let updated = state.long_drip.update(&id, |mut a| {
        a.reviewer_id = Some(reviewer.clone());
        a.reviewed_at = Some(now.clone());
        if escalated {
            a.escalated_to_commissioner = true;
            a.escalated_at = Some(now.clone());
        }
        a
    });
    match updated {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no long-drip alert with id {id}"),
        )),
    }
}

pub async fn next_cycle_handoff(State(state): State<SharedState>) -> impl IntoResponse {
    info!("next_cycle_handoff");
    // Real. Previously returned an empty handoff_items array. The handoff is
    // derived from findings that are still open, so it reflects what is actually
    // outstanding rather than nothing.
    let all_findings = state.findings_by_id.list();
    let mut open: Vec<&Finding> = all_findings
        .iter()
        .filter(|f| f.status == FindingStatus::Submitted)
        .collect();
    open.sort_by(|a, b| a.submitted_at.cmp(&b.submitted_at));
    let items: Vec<serde_json::Value> = open
        .iter()
        .map(|f| serde_json::json!({
            "finding_id": f.id,
            "election_cycle": f.election_cycle,
            "severity": f.severity,
            "status": f.status,
            "is_long_drip": f.is_long_drip,
            "submitted_at": f.submitted_at,
        }))
        .collect();
    Json(serde_json::json!({ "handoff_items": items })).into_response()
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
