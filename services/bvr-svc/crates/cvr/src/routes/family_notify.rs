use crate::{error::ApiError, models::FamilyNotificationRequest, state::AppState};
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};


pub fn router() -> Router<AppState> {
    Router::new().route("/v1/cvr/family-notify", post(family_notify))
}

async fn family_notify(
    State(_state): State<AppState>,
    Json(_req): Json<FamilyNotificationRequest>,
) -> Result<StatusCode, ApiError> {
    // Persistence prerequisite: FamilyNotificationRequest carries a hashed
    // voter id (SHA-256 hex), but eel_voter_register_events.voter_id is UUID
    // NOT NULL and the voters table (V1) keys on TEXT voter_id with the
    // national id stored encrypted, so there is no lookup path from the hash
    // to an event row. The bvr-svc service migrations (owner: bvr-svc
    // maintainers; next free slot V054) must add either a hashed-voter-id
    // index or a dedicated eel_family_notifications table before the
    // FAMILY_NOTIFICATION / FAMILY_SHORTCODE event can be written here.
    Ok(StatusCode::ACCEPTED)
}
