use axum::{routing::get, Json, Router};
use serde_json::json;

pub fn router() -> Router {
    Router::new().route("/v1/cvr/events", get(list_events))
}

async fn list_events() -> Json<serde_json::Value> {
    // TODO: query DB for voter_register_event records
    Json(json!({"events": []}))
}
