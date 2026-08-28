use crate::handlers::*;
use crate::state::SharedState;
use axum::{routing::get, Router};

pub fn build_router(state: SharedState) -> Router {
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
    // Public contract endpoints (governance-intl-observer/coordination.yaml)
        .route(
            "/api/v1/governance/intl-observer/missions",
            get(list_observer_missions).post(register_observer_mission),
        )
        .route(
            "/api/v1/governance/intl-observer/missions/{mission_id}",
            get(get_observer_mission),
        )
        .route(
            "/api/v1/governance/intl-observer/missions/{mission_id}/reports",
            get(list_observer_reports).post(submit_observer_report),
        )
        .with_state(state)
}
