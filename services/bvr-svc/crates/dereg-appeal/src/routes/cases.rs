use axum::{
    extract::Path,
    http::StatusCode,
    routing::get,
    Json,
    Router,
};
use serde_json::json;
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/v1/dereg/cases", get(list_cases).post(create_case))
        .route("/v1/dereg/cases/:id", get(get_case).patch(patch_case))
}

async fn list_cases() -> Json<serde_json::Value> {
    Json(json!({"cases": []}))
}

async fn create_case() -> (StatusCode, Json<serde_json::Value>) {
    // TODO: parse body, validate m35_event_id, create DeregistrationCase
    (StatusCode::CREATED, Json(json!({"status": "created"})))
}

async fn get_case(Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    Json(json!({"id": id}))
}

async fn patch_case(Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    Json(json!({"id": id, "updated": true}))
}
