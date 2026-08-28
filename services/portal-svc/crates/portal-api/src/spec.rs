//! Public contract handlers (portal/public.yaml).

use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// GET /api/v1/portal/results — national results.
pub async fn get_national_results() -> impl IntoResponse {
    Json(json!({ "results": [], "generated_at": chrono::Utc::now().to_rfc3339() }))
}

/// GET /api/v1/portal/results/{scope}/{code} — scoped results.
pub async fn get_scoped_results(Path((scope, code)): Path<(String, String)>) -> impl IntoResponse {
    Json(json!({ "scope": scope, "code": code, "results": [] }))
}

/// GET /api/v1/portal/forms/{form_id}/proof — form inclusion proof.
pub async fn get_form_inclusion_proof(Path(form_id): Path<String>) -> impl IntoResponse {
    Json(json!({ "form_id": form_id, "verified": false, "proof": serde_json::Value::Null }))
}

/// GET /api/v1/portal/merkle/{root_id} — merkle root.
pub async fn get_merkle_root(Path(root_id): Path<String>) -> impl IntoResponse {
    Json(json!({ "root_id": root_id, "root_hash": "0".repeat(64), "tree_size": 0 }))
}

/// GET /api/v1/portal/turnout — live turnout.
pub async fn get_live_turnout() -> impl IntoResponse {
    Json(json!({ "turnout_pct": 0.0, "as_of": chrono::Utc::now().to_rfc3339() }))
}

/// GET /api/v1/portal/civic/{item_id} — civic content.
pub async fn get_civic_content(Path(item_id): Path<String>) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "detail": format!("civic item {item_id} not found") })),
    )
}

/// POST /api/v1/portal/admin/content — publish civic content.
pub async fn publish_civic_content(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    (
        StatusCode::CREATED,
        Json(json!({ "content_id": uuid::Uuid::now_v7().to_string(), "request": req })),
    )
}
