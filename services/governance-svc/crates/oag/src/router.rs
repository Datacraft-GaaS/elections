use crate::handlers::*;
use crate::state::SharedState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/oag/engagements",
            get(list_engagements).post(create_engagement),
        )
        .route("/oag/engagements/{engagement_id}", get(get_engagement))
        .route(
            "/oag/engagements/{engagement_id}/evidence",
            post(extract_evidence),
        )
        .route(
            "/oag/engagements/{engagement_id}/response",
            post(record_response),
        )
        .route("/oag/engagements/{engagement_id}/response", get(get_response))
        .route(
            "/oag/published-reports",
            get(list_published_reports).post(register_published_report),
        )
        .with_state(state)
}
