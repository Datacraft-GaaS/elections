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

/// The digest of the canonical report body, `sha256:`-prefixed the way the rest
/// of the tree writes payload hashes.
fn body_digest(report_id: &str, period: &str, report_type: &ReportType) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    for part in [
        report_id,
        period,
        &serde_json::to_string(report_type).unwrap_or_default(),
    ] {
        h.update(part.as_bytes());
        h.update(b"\x1e");
    }
    format!("sha256:{}", hex::encode(h.finalize()))
}

pub async fn list_reports(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_reports");
    Json(serde_json::json!({ "reports": state.statutory_reports_sorted() }))
}

pub async fn generate_report(
    State(state): State<SharedState>,
    Json(req): Json<GenerateStatutoryReportRequest>,
) -> axum::response::Response {
    info!(period = %req.period, "generate_report");
    if req.period.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "period is required");
    }
    let report_id = uuid::Uuid::now_v7().to_string();
    let report_type = req.report_type.unwrap_or_default();
    let report = StatutoryReport {
        content_hash: body_digest(&report_id, &req.period, &report_type),
        report_id: report_id.clone(),
        report_type,
        // The lifecycle starts at `Generating`, not `Ready`: a report with no
        // body behind it is not ready for testimony or handover, and the
        // downstream handlers gate on this field.
        status: ReportStatus::Generating,
        period: req.period,
        // Nothing here is signed -- there is no signing key reachable from this
        // service -- and the report body is assembled by the caller, not here.
        blocking_reason: Some(
            "the report body is assembled outside this service; it records the \
             request and its digest but cannot render statutory content without \
             the reporting data warehouse, which is not in this workspace"
                .to_string(),
        ),
        generated_at: chrono::Utc::now().to_rfc3339(),
    };
    state
        .statutory_reports
        .lock()
        .expect("lock poisoned")
        .insert(report_id, report.clone());
    (StatusCode::CREATED, Json(report)).into_response()
}

pub async fn get_report(
    State(state): State<SharedState>,
    Path(report_id): Path<String>,
) -> axum::response::Response {
    info!(report_id, "get_report");
    let guard = state.statutory_reports.lock().expect("lock poisoned");
    match guard.get(&report_id) {
        Some(r) => Json(r.clone()).into_response(),
        // Previously any id returned a blank StatutoryReport with 200, which
        // reads as a real report with no content.
        None => problem(
            StatusCode::NOT_FOUND,
            format!("no statutory report with id {report_id}"),
        ),
    }
}

/// Mark a report ready, which is what opens it to testimony, handover and
/// pre-registration.
///
/// This is the transition the gating rule needs: a report becomes `Ready` when its
/// body exists, and that is recorded as a fact about the report rather than
/// assumed at read time.
pub async fn mark_report_ready(
    State(state): State<SharedState>,
    Path(report_id): Path<String>,
) -> axum::response::Response {
    info!(report_id, "mark_report_ready");
    let mut guard = state.statutory_reports.lock().expect("lock poisoned");
    let Some(report) = guard.get_mut(&report_id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no statutory report with id {report_id}"),
        );
    };
    if report.status == ReportStatus::Delivered {
        return problem(
            StatusCode::CONFLICT,
            "a delivered report cannot return to generating",
        );
    }
    report.status = ReportStatus::Ready;
    // The blocking reason described generation; readiness is the caller's
    // assertion that the body now exists, so it no longer applies.
    report.blocking_reason = None;
    let report = report.clone();
    drop(guard);
    Json(report).into_response()
}

pub async fn create_testimony_workspace(
    State(state): State<SharedState>,
    Json(req): Json<OpenTestimonyWorkspaceRequest>,
) -> axum::response::Response {
    info!(report_id = %req.report_id, "create_testimony_workspace");
    if req.assignee.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "assignee is required");
    }
    let Some(status) = state.report_status(&req.report_id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no statutory report with id {}", req.report_id),
        );
    };
    if !status.allows_downstream() {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "report is {}; testimony requires a ready report",
                status.wire()
            ),
        );
    }
    let workspace = TestimonyWorkspace {
        workspace_id: uuid::Uuid::now_v7().to_string(),
        report_id: req.report_id,
        assignee: req.assignee,
        created_at: chrono::Utc::now().to_rfc3339(),
        status: "open".to_string(),
    };
    state
        .workspaces
        .lock()
        .expect("lock poisoned")
        .insert(workspace.workspace_id.clone(), workspace.clone());
    (StatusCode::CREATED, Json(workspace)).into_response()
}

pub async fn list_testimony_workspaces(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_testimony_workspaces");
    Json(serde_json::json!({ "workspaces": state.workspaces_sorted() }))
}

pub async fn record_judiciary_handover(
    State(state): State<SharedState>,
    Json(req): Json<JudiciaryHandoverRequest>,
) -> axum::response::Response {
    info!(report_id = %req.report_id, "record_judiciary_handover");
    if req.recipient_court.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "recipient_court is required");
    }
    let Some(status) = state.report_status(&req.report_id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no statutory report with id {}", req.report_id),
        );
    };
    if !status.allows_downstream() {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "report is {}; a report cannot be handed to a court before it is ready",
                status.wire()
            ),
        );
    }
    let handover = JudiciaryHandover {
        handover_id: uuid::Uuid::now_v7().to_string(),
        report_id: req.report_id,
        recipient_court: req.recipient_court,
        // A handover is identified by the report and court it concerns; there is
        // no external docket to cite from here, so the id is the reference and
        // is not dressed up as one.
        handover_ref: uuid::Uuid::now_v7().to_string(),
        handed_over_at: chrono::Utc::now().to_rfc3339(),
    };
    // Delivering a report to a court moves it to Delivered.
    if let Some(report) = state
        .statutory_reports
        .lock()
        .expect("lock poisoned")
        .get_mut(&handover.report_id)
    {
        report.status = ReportStatus::Delivered;
    }
    state
        .handovers
        .lock()
        .expect("lock poisoned")
        .insert(handover.handover_id.clone(), handover.clone());
    (StatusCode::CREATED, Json(handover)).into_response()
}

pub async fn register_sc_it(
    State(state): State<SharedState>,
    Json(req): Json<ScPreregistrationRequest>,
) -> axum::response::Response {
    info!(report_id = %req.report_id, "register_sc_it");
    if req.sc_it_contact.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "sc_it_contact is required");
    }
    let Some(status) = state.report_status(&req.report_id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no statutory report with id {}", req.report_id),
        );
    };
    if !status.allows_downstream() {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "report is {}; pre-registration requires a ready report",
                status.wire()
            ),
        );
    }
    let prereg = ScPreregistration {
        prereg_id: uuid::Uuid::now_v7().to_string(),
        report_id: req.report_id,
        sc_it_contact: req.sc_it_contact,
        registered_at: chrono::Utc::now().to_rfc3339(),
        // No SC/IT committee endpoint exists in this workspace to acknowledge a
        // registration, so there is no confirmation reference to record. Left
        // null rather than filled with an id that implies one was issued.
        confirmation_ref: None,
    };
    state
        .preregs
        .lock()
        .expect("lock poisoned")
        .insert(prereg.prereg_id.clone(), prereg.clone());
    (StatusCode::CREATED, Json(prereg)).into_response()
}
// ---------------------------------------------------------------------------
// Public contract endpoints (governance-reporting/reports.yaml)
// ---------------------------------------------------------------------------

/// GET /api/v1/governance/reporting/parliamentary-reports — list report jobs.
pub async fn list_parliamentary_reports(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_parliamentary_reports");
    let guard = state.reports.lock().expect("lock poisoned");
    let mut items: Vec<spec::ReportJob> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }))
}

/// POST /api/v1/governance/reporting/parliamentary-reports — generate a report.
pub async fn generate_parliamentary_report(
    State(state): State<SharedState>,
    Json(req): Json<spec::ReportRequest>,
) -> impl IntoResponse {
    info!(report_type = %req.report_type, "generate_parliamentary_report");
    let now = chrono::Utc::now().to_rfc3339();
    let job = spec::ReportJob {
        job_id: uuid::Uuid::now_v7().to_string(),
        period_start: req.period_start,
        period_end: req.period_end,
        report_type: req.report_type,
        scope: req.scope,
        status: "queued".to_string(),
        download_url: None,
        created_at: now.clone(),
        updated_at: now,
    };
    let mut guard = state.reports.lock().expect("lock poisoned");
    guard.insert(job.job_id.clone(), job.clone());
    drop(guard);
    (StatusCode::ACCEPTED, Json(job)).into_response()
}

/// GET /api/v1/governance/reporting/parliamentary-reports/{report_id} — fetch one.
pub async fn get_parliamentary_report(
    State(state): State<SharedState>,
    Path(report_id): Path<String>,
) -> impl IntoResponse {
    info!(report_id, "get_parliamentary_report");
    let guard = state.reports.lock().expect("lock poisoned");
    match guard.get(&report_id) {
        Some(r) => Json(r.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "report not found"})),
        )
            .into_response(),
    }
}

/// POST /api/v1/governance/reporting/oag-submissions — submit a report to OAG.
pub async fn submit_to_oag(
    State(state): State<SharedState>,
    Json(req): Json<spec::OagSubmissionRequest>,
) -> impl IntoResponse {
    info!(report_id = %req.report_id, "submit_to_oag");
    let now = chrono::Utc::now().to_rfc3339();
    let submission = spec::OagSubmission {
        submission_id: uuid::Uuid::now_v7().to_string(),
        report_id: req.report_id,
        oag_ref: req.oag_ref,
        submission_type: req.submission_type,
        submitted_at: req.submitted_at,
        status: "submitted".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    let mut guard = state.oag_submissions.lock().expect("lock poisoned");
    guard.insert(submission.submission_id.clone(), submission.clone());
    drop(guard);
    (StatusCode::CREATED, Json(submission)).into_response()
}

/// GET /api/v1/governance/reporting/oag-submissions — list OAG submissions.
pub async fn list_oag_submissions(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_oag_submissions");
    let guard = state.oag_submissions.lock().expect("lock poisoned");
    let mut items: Vec<spec::OagSubmission> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }))
}
