use crate::handlers::*;
use axum::{routing::get, Router};

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/v1/observer-missions",
            get(list_missions).post(register_mission),
        )
        .route(
            "/v1/observer-missions/{id}/briefings",
            get(list_briefings).post(schedule_briefing),
        )
        .route(
            "/v1/observer-missions/{id}/reports",
            get(list_reports).post(submit_report),
        )
        .route(
            "/v1/joint-statements",
            get(list_joint_statements).post(publish_joint_statement),
        )
}
