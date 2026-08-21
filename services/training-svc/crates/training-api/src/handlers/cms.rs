use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tracing::info;
use uuid::Uuid;
use crate::models::*;

pub async fn post_content(Json(req): Json<ContentItemCreateReq>) -> impl IntoResponse {
    info!(title=%req.title, language=%req.language, "post_content");
    (StatusCode::CREATED, Json(ContentItem {
        id: Uuid::new_v4(),
        title: req.title,
        language: req.language,
        format: req.format,
        topic_category: req.topic_category,
        status: "draft".to_string(),
        expiry_date: None,
    }))
}

pub async fn get_content_list() -> impl IntoResponse {
    Json(serde_json::json!({ "items": [] }))
}

pub async fn get_content_public() -> impl IntoResponse {
    Json(serde_json::json!({ "items": [] }))
}

pub async fn submit_fact_check(
    Path(id): Path<Uuid>,
    Json(req): Json<FactCheckDecisionReq>,
) -> impl IntoResponse {
    info!(content_id=%id, decision=%req.decision, "submit_fact_check");
    Json(serde_json::json!({ "id": id, "decision": req.decision, "status": "reviewed" }))
}

pub async fn approve_content(Path(id): Path<Uuid>) -> impl IntoResponse {
    info!(content_id=%id, "approve_content");
    Json(serde_json::json!({ "id": id, "status": "approved" }))
}
