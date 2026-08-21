pub mod applications;
pub mod handoff;
pub mod health;
pub mod otp;

use axum::{routing::get, routing::post, Router};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/vprs/v1/applications", post(applications::create_application))
        .route(
            "/vprs/v1/applications/:id/status",
            get(applications::get_status),
        )
        .route(
            "/vprs/v1/applications/:id",
            axum::routing::delete(applications::cancel_application),
        )
        .route("/vprs/v1/handoff/:token", get(handoff::get_handoff))
        .route("/vprs/v1/otp/request", post(otp::request_otp))
        .route("/vprs/v1/otp/verify", post(otp::verify_otp))
        .with_state(state)
}
