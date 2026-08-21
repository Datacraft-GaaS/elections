use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(list_items), info(title = "{{project-name}} API", version = "0.1.0"))]
pub struct ApiDoc;

pub fn router() -> Router {
	Router::new().route("/api/v1/items", get(list_items))
}

/// List items.
#[utoipa::path(get, path = "/api/v1/items", responses((status = 200, description = "OK")))]
async fn list_items() -> Json<Value> {
	Json(json!({ "items": [] }))
}
