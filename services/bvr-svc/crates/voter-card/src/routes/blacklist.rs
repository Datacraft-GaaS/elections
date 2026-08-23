use axum::{extract::State, http::StatusCode, routing::post, Json, Router};

use crate::{blacklist::BlacklistService, error::ApiError, models::BlacklistEntry, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/voter-cards/blacklist", post(add_blacklist))
}

async fn add_blacklist(
    State(state): State<AppState>,
    Json(entry): Json<BlacklistEntry>,
) -> Result<StatusCode, ApiError> {
    let svc = BlacklistService;
    svc.add_entry(&state.pool, entry)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(StatusCode::CREATED)
}
