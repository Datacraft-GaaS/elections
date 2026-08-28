use crate::handlers::*;
use crate::state::SharedState;
use axum::{
    routing::{get, patch, post},
    Router,
};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/v1/review/findings",
            post(submit_finding).get(list_findings),
        )
        .route("/v1/review/findings/{id}", patch(triage_finding))
        .route("/v1/review/recommendations", get(list_recommendations))
        .route(
            "/v1/review/recommendations/{id}/sign",
            post(sign_recommendation),
        )
        .route("/v1/review/public", get(public_tracker))
        .route(
            "/v1/review/parliament-referrals",
            post(create_parliament_referral),
        )
        .route(
            "/v1/review/parliament-referrals/{id}",
            patch(record_parliament_response),
        )
        .route("/v1/review/long-drip", get(list_long_drip))
        .route(
            "/v1/review/long-drip/{id}/review",
            post(record_long_drip_review),
        )
        .route("/v1/review/next-cycle-handoff", get(next_cycle_handoff))
        // Public contract endpoints (governance-review/post-election.yaml)
        .route(
            "/api/v1/governance/review/reviews",
            get(list_post_election_reviews).post(initiate_post_election_review),
        )
        .route(
            "/api/v1/governance/review/reviews/{review_id}",
            get(get_post_election_review),
        )
        .route(
            "/api/v1/governance/review/reviews/{review_id}/findings",
            post(add_review_finding),
        )
        .route(
            "/api/v1/governance/review/reviews/{review_id}/publish",
            post(publish_review_report),
        )
        .with_state(state)
}
