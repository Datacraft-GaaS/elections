use axum::{routing::get, Json, Router};
use serde_json::json;

pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}

async fn healthz() -> Json<serde_json::Value> {
    Json(json!({"ok": true}))
}

async fn readyz() -> Json<serde_json::Value> {
    Json(json!({"ready": true}))
}
