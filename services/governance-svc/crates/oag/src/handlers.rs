use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use tracing::info;

use crate::models::*;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn list_engagements() -> impl IntoResponse {
    info!("list_engagements");
    let engagements: Vec<OagEngagement> = vec![];
    Json(serde_json::json!({ "engagements": engagements }))
}

pub async fn create_engagement() -> impl IntoResponse {
    info!("create_engagement");
    Json(OagEngagement::default())
}

pub async fn get_engagement(Path(engagement_id): Path<String>) -> impl IntoResponse {
    info!(engagement_id, "get_engagement");
    Json(OagEngagement { engagement_id, ..Default::default() })
}

pub async fn extract_evidence(Path(engagement_id): Path<String>) -> impl IntoResponse {
    info!(engagement_id, "extract_evidence");
    Json(EvidenceExtract { engagement_id, ..Default::default() })
}

pub async fn record_response(Path(engagement_id): Path<String>) -> impl IntoResponse {
    info!(engagement_id, "record_response");
    Json(OagResponse { engagement_id, ..Default::default() })
}

pub async fn list_published_reports() -> impl IntoResponse {
    info!("list_published_reports");
    let reports: Vec<PublishedReport> = vec![];
    Json(serde_json::json!({ "reports": reports }))
}

pub async fn register_published_report() -> impl IntoResponse {
    info!("register_published_report");
    Json(PublishedReport::default())
}
