use crate::state::AppState;
use axum::{routing::get, Json, Router};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/dereg/reports", get(list_reports))
}

async fn list_reports() -> Json<serde_json::Value> {
    Json(json!({"reports": []}))
}
