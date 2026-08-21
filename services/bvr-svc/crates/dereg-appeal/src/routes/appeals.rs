use axum::{http::StatusCode, routing::post, Json, Router};
use serde_json::json;

pub fn router() -> Router {
    Router::new().route("/v1/dereg/appeals", post(file_appeal))
}

async fn file_appeal() -> (StatusCode, Json<serde_json::Value>) {
    // TODO: parse AppealRecord, validate case status allows appeal, persist
    (StatusCode::CREATED, Json(json!({"status": "appeal_filed"})))
}
