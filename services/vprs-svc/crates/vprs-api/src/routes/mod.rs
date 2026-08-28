pub mod applications;
pub mod handoff;
pub mod health;
pub mod otp;
pub mod spec;

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
        // Public contract endpoints (vprs/portal.yaml)
        .route("/api/v1/vprs/applications", post(applications::create_application))
        .route("/api/v1/vprs/applications/{app_id}", get(applications::get_status).delete(applications::cancel_application))
        .route("/api/v1/vprs/handoff/{token}", get(handoff::get_handoff))
        .route("/api/v1/vprs/otp/request", post(otp::request_otp))
        .route("/api/v1/vprs/otp/verify", post(otp::verify_otp))
        .route("/api/v1/vprs/admin/analytics", get(spec::get_vprs_analytics))
        .with_state(state)
}
