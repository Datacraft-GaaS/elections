use axum::{routing::get, Json, Router};
use serde_json::json;

pub fn router() -> Router<crate::state::AppState> {
    Router::new().route("/healthz", get(healthz))
}

async fn healthz() -> Json<serde_json::Value> {
    Json(json!({"ok": true}))
}
