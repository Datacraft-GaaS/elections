use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use tracing::info;

use crate::models::*;

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
