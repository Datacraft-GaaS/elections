use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/v1/voter-cards/batches", post(create_batch))
        .route("/v1/voter-cards/batches/:id", get(get_batch))
}

async fn create_batch() -> (StatusCode, Json<serde_json::Value>) {
    // TODO: parse CardBatch, validate election_id + constituency_id, persist
    (StatusCode::CREATED, Json(json!({"status": "batch_created"})))
}

async fn get_batch(Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    Json(json!({"id": id}))
}
