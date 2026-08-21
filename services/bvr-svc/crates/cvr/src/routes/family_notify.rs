use axum::{http::StatusCode, routing::post, Json, Router};

use crate::{error::ApiError, models::FamilyNotificationRequest};

pub fn router() -> Router {
    Router::new().route("/v1/cvr/family-notify", post(family_notify))
}

async fn family_notify(
    Json(_req): Json<FamilyNotificationRequest>,
) -> Result<StatusCode, ApiError> {
    // TODO: validate *400# shortcode, hash voter_id, create FamilyNotification event.
    Ok(StatusCode::ACCEPTED)
}
