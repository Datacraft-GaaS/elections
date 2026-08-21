use axum::{routing::get, Json, Router};
use serde_json::json;

pub fn router() -> Router {
    Router::new().route("/v1/voter-cards/status", get(card_status))
}

async fn card_status() -> Json<serde_json::Value> {
    // TODO: query card status by card_number or voter_id
    Json(json!({"status": "ok"}))
}
