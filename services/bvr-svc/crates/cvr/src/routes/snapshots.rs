use axum::{routing::get, Json, Router};
use serde_json::json;

pub fn router() -> Router {
    Router::new().route("/v1/cvr/snapshots", get(list_snapshots))
}

async fn list_snapshots() -> Json<serde_json::Value> {
    // TODO: query DB for register_snapshot records
    Json(json!({"snapshots": []}))
}
