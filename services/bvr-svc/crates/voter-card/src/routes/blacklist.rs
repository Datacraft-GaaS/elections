use axum::{http::StatusCode, routing::post, Json, Router};

use crate::{blacklist::BlacklistService, error::ApiError, models::BlacklistEntry};

pub fn router() -> Router {
    Router::new().route("/v1/voter-cards/blacklist", post(add_blacklist))
}

async fn add_blacklist(
    Json(entry): Json<BlacklistEntry>,
) -> Result<StatusCode, ApiError> {
    let svc = BlacklistService::new();
    svc.add_entry(entry)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(StatusCode::CREATED)
}
