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

pub async fn list_reports() -> impl IntoResponse {
    info!("list_reports");
    let reports: Vec<StatutoryReport> = vec![];
    Json(serde_json::json!({ "reports": reports }))
}

pub async fn generate_report() -> impl IntoResponse {
    info!("generate_report");
    Json(StatutoryReport::default())
}

pub async fn get_report(Path(report_id): Path<String>) -> impl IntoResponse {
    info!(report_id, "get_report");
    Json(StatutoryReport { report_id, ..Default::default() })
}

pub async fn create_testimony_workspace() -> impl IntoResponse {
    info!("create_testimony_workspace");
    Json(TestimonyWorkspace::default())
}

pub async fn list_testimony_workspaces() -> impl IntoResponse {
    info!("list_testimony_workspaces");
    let workspaces: Vec<TestimonyWorkspace> = vec![];
    Json(serde_json::json!({ "workspaces": workspaces }))
}

pub async fn record_judiciary_handover() -> impl IntoResponse {
    info!("record_judiciary_handover");
    Json(JudiciaryHandover::default())
}

pub async fn register_sc_it() -> impl IntoResponse {
    info!("register_sc_it");
    Json(ScPreregistration::default())
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
