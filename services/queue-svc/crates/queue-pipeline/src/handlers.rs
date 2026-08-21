use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use tracing::info;

use crate::models::SignatureFusionInput;
use crate::pipeline::fuse_snapshot;

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"ok": true})))
}

pub async fn post_snapshot(
    Path(station_code): Path<String>,
    Json(input): Json<SignatureFusionInput>,
) -> impl IntoResponse {
    info!(station_code, "post_snapshot");
    let snapshot = fuse_snapshot(&station_code, input);
    (StatusCode::OK, Json(snapshot))
}
