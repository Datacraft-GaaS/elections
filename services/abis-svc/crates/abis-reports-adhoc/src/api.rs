use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::*;

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<sqlx::PgPool>,
}

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListReportsParams {
    pub visibility: Option<String>,
    pub include_shared: Option<bool>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn list_reports(
    State(_state): State<AppState>,
    Query(_params): Query<ListReportsParams>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "reports": [] }))
}

async fn create_report(
    State(_state): State<AppState>,
    Json(_body): Json<CreateReportRequest>,
) -> impl IntoResponse {
    (StatusCode::CREATED, Json(serde_json::json!({ "created": true })))
}

async fn update_report(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<UpdateReportRequest>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "updated": true }))
}

async fn archive_report(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

async fn list_versions(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "versions": [] }))
}

async fn create_version(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<CreateVersionRequest>,
) -> impl IntoResponse {
    (StatusCode::CREATED, Json(serde_json::json!({ "created": true })))
}

async fn get_version(
    State(_state): State<AppState>,
    Path(_version_id): Path<Uuid>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "version": null }))
}

async fn run_preview(
    State(_state): State<AppState>,
    Path(_version_id): Path<Uuid>,
    Json(_body): Json<PreviewRequest>,
) -> impl IntoResponse {
    (StatusCode::CREATED, Json(serde_json::json!({ "preview_id": Uuid::now_v7() })))
}

async fn get_preview(
    State(_state): State<AppState>,
    Path(_preview_id): Path<Uuid>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "preview": null }))
}

async fn publish_version(
    State(_state): State<AppState>,
    Path(_version_id): Path<Uuid>,
    Json(_body): Json<PublishRequest>,
) -> impl IntoResponse {
    (StatusCode::CREATED, Json(serde_json::json!({ "definition_id": Uuid::now_v7() })))
}

async fn grant_share(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<ShareRequest>,
) -> impl IntoResponse {
    (StatusCode::CREATED, Json(serde_json::json!({ "share_id": Uuid::now_v7() })))
}

async fn revoke_share(
    State(_state): State<AppState>,
    Path(_share_id): Path<Uuid>,
) -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

async fn list_snippets(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "snippets": [] }))
}

async fn create_snippet(
    State(_state): State<AppState>,
    Json(_body): Json<CreateSnippetRequest>,
) -> impl IntoResponse {
    (StatusCode::CREATED, Json(serde_json::json!({ "created": true })))
}

async fn delete_snippet(
    State(_state): State<AppState>,
    Path(_snippet_id): Path<Uuid>,
) -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/reports", get(list_reports).post(create_report))
        .route("/reports/{id}", patch(update_report).delete(archive_report))
        .route("/reports/{id}/versions", get(list_versions).post(create_version))
        .route("/versions/{version_id}", get(get_version))
        .route("/versions/{version_id}/preview", post(run_preview))
        .route("/previews/{preview_id}", get(get_preview))
        .route("/versions/{version_id}/publish", post(publish_version))
        .route("/reports/{id}/shares", post(grant_share))
        .route("/shares/{share_id}", delete(revoke_share))
        .route("/snippets", get(list_snippets).post(create_snippet))
        .route("/snippets/{snippet_id}", delete(delete_snippet))
        .with_state(state)
}
