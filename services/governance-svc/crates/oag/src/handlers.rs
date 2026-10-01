use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::models::*;
use crate::state::SharedState;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

/// A problem+json response.
fn problem(status: StatusCode, detail: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail.into(),
        })),
    )
        .into_response()
}

pub async fn list_engagements(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_engagements");
    Json(serde_json::json!({ "engagements": state.engagements_sorted() }))
}

pub async fn create_engagement(
    State(state): State<SharedState>,
    Json(req): Json<CreateEngagementRequest>,
) -> axum::response::Response {
    info!(scope = %req.scope, "create_engagement");
    if req.scope.trim().is_empty() || req.oag_ref.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "scope and oag_ref are required",
        );
    }
    let engagement = OagEngagement {
        engagement_id: uuid::Uuid::now_v7().to_string(),
        scope: req.scope,
        // A new engagement opens in scoping: the scope is agreed before the
        // audit begins.
        status: EngagementStatus::Scoping,
        initiated_at: chrono::Utc::now().to_rfc3339(),
        oag_ref: req.oag_ref,
    };
    state
        .engagements
        .lock()
        .expect("lock poisoned")
        .insert(engagement.engagement_id.clone(), engagement.clone());
    (StatusCode::CREATED, Json(engagement)).into_response()
}

pub async fn get_engagement(
    State(state): State<SharedState>,
    Path(engagement_id): Path<String>,
) -> axum::response::Response {
    info!(engagement_id, "get_engagement");
    let guard = state.engagements.lock().expect("lock poisoned");
    match guard.get(&engagement_id) {
        Some(e) => Json(e.clone()).into_response(),
        None => problem(
            StatusCode::NOT_FOUND,
            format!("no engagement with id {engagement_id}"),
        ),
    }
}

pub async fn extract_evidence(
    State(state): State<SharedState>,
    Path(engagement_id): Path<String>,
    Json(req): Json<ExtractEvidenceRequest>,
) -> axum::response::Response {
    info!(engagement_id, "extract_evidence");
    let guard = state.engagements.lock().expect("lock poisoned");
    let Some(engagement) = guard.get(&engagement_id).cloned() else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no engagement with id {engagement_id}"),
        );
    };
    // Previously any id produced an extract with the id echoed back, so evidence
    // could be extracted from an engagement that did not exist.
    if !engagement.status.accepts_evidence() {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "engagement is {}; evidence cannot be extracted once that is the case",
                engagement.status.wire()
            ),
        );
    }
    if req.description.trim().is_empty() || req.extracted_by.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "description and extracted_by are required",
        );
    }
    // The extract commits to the engagement id, the description, and the
    // extractor, so the digest covers who extracted what from where.
    let digest = payload_digest(&engagement_id, &req.description, &req.extracted_by);
    let extract = EvidenceExtract {
        extract_id: uuid::Uuid::now_v7().to_string(),
        engagement_id: engagement_id.clone(),
        description: req.description,
        payload_hash: digest,
        extracted_at: chrono::Utc::now().to_rfc3339(),
        extracted_by: req.extracted_by,
    };
    drop(guard);
    state
        .extracts
        .lock()
        .expect("lock poisoned")
        .insert(extract.extract_id.clone(), extract.clone());
    (StatusCode::CREATED, Json(extract)).into_response()
}

/// The SHA-256 over the extract's identifying fields, `sha256:`-prefixed the way
/// the rest of the tree writes payload hashes.
fn payload_digest(engagement_id: &str, description: &str, extracted_by: &str) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    for part in [engagement_id, description, extracted_by] {
        h.update(part.as_bytes());
        h.update(b"\x1e");
    }
    format!("sha256:{}", hex::encode(h.finalize()))
}

pub async fn record_response(
    State(state): State<SharedState>,
    Path(engagement_id): Path<String>,
    Json(req): Json<RecordResponseRequest>,
) -> axum::response::Response {
    info!(engagement_id, "record_response");
    let mut guard = state.engagements.lock().expect("lock poisoned");
    let Some(engagement) = guard.get(&engagement_id).cloned() else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no engagement with id {engagement_id}"),
        );
    };
    // One response per engagement. Previously a response could be recorded
    // repeatedly and each call returned a different id for the same question.
    if !engagement.status.accepts_response() {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "engagement is {}; the OAG has already responded",
                engagement.status.wire()
            ),
        );
    }
    if req.summary.trim().is_empty() || req.disposition.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "summary and disposition are required",
        );
    }
    let response = OagResponse {
        response_id: uuid::Uuid::now_v7().to_string(),
        engagement_id: engagement_id.clone(),
        summary: req.summary,
        disposition: req.disposition,
        responded_at: chrono::Utc::now().to_rfc3339(),
        oag_signatory: req.oag_signatory,
    };
    // Recording the response closes the question on the engagement itself.
    let updated = OagEngagement {
        status: EngagementStatus::Responded,
        ..engagement
    };
    guard.insert(engagement_id, updated);
    drop(guard);
    state
        .responses
        .lock()
        .expect("lock poisoned")
        .insert(response.response_id.clone(), response.clone());
    (StatusCode::CREATED, Json(response)).into_response()
}

/// The response recorded against an engagement, if any.
pub async fn get_response(
    State(state): State<SharedState>,
    Path(engagement_id): Path<String>,
) -> axum::response::Response {
    info!(engagement_id, "get_response");
    let guard = state.engagements.lock().expect("lock poisoned");
    if !guard.contains_key(&engagement_id) {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no engagement with id {engagement_id}"),
        );
    }
    drop(guard);
    let guard = state.responses.lock().expect("lock poisoned");
    match guard.values().find(|r| r.engagement_id == engagement_id) {
        Some(r) => Json(r.clone()).into_response(),
        None => problem(
            StatusCode::NOT_FOUND,
            format!("no response has been recorded for engagement {engagement_id}"),
        ),
    }
}

pub async fn list_published_reports(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_published_reports");
    Json(serde_json::json!({ "reports": state.published_reports_sorted() }))
}

pub async fn register_published_report(
    State(state): State<SharedState>,
    Json(req): Json<RegisterPublishedReportRequest>,
) -> axum::response::Response {
    info!(report_type = %req.report_type, "register_published_report");
    if req.title.trim().is_empty() || req.uri.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "title and uri are required");
    }
    // `hash` is the digest of the report contents, which this service does not
    // hold -- it is published elsewhere. A caller may supply it; when they do
    // not, the field is null rather than a digest of the metadata, which would
    // look like a content hash without being one.
    let report = PublishedReport {
        pub_report_id: uuid::Uuid::now_v7().to_string(),
        title: req.title,
        report_type: req.report_type,
        published_at: chrono::Utc::now().to_rfc3339(),
        uri: req.uri,
        hash: req.hash,
    };
    state
        .published_reports
        .lock()
        .expect("lock poisoned")
        .insert(report.pub_report_id.clone(), report.clone());
    (StatusCode::CREATED, Json(report)).into_response()
}
