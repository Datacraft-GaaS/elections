use axum::{extract::Path, routing::post, Json, Router};
use serde::Serialize;

use crate::error::ApiError;

#[derive(Debug, Serialize)]
pub struct HandoffResponse {
    pub token: String,
    pub status: String,
    pub voter_id: Option<String>,
}

pub fn router() -> Router {
    Router::new().route("/api/v1/bvr/handoff/{token}", post(process_handoff))
}

async fn process_handoff(
    Path(token): Path<String>,
) -> Result<Json<HandoffResponse>, ApiError> {
    Ok(Json(HandoffResponse {
        token,
        status: "redeemed".into(),
        voter_id: None,
    }))
}
