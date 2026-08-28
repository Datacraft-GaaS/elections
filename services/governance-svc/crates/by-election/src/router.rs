use crate::handlers::*;
use crate::state::SharedState;
use axum::{
    routing::{get, post, put},
    Router,
};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        // M50 by-election endpoints
        .route(
            "/v1/by-elections",
            get(list_by_elections).post(create_by_election),
        )
        .route("/v1/by-elections/{id}", get(get_by_election))
        .route(
            "/v1/by-elections/{id}/register-subset",
            post(generate_register_subset),
        )
        .route("/v1/by-elections/{id}/kit-plan", post(create_kit_plan))
        .route("/v1/by-elections/{id}/activate", put(activate_by_election))
        .route(
            "/v1/by-elections/{id}/cost-report",
            post(submit_cost_report).get(get_cost_report),
        )
    // Public contract endpoints (governance-by-election/operations.yaml)
        .route(
            "/api/v1/governance/by-election/elections",
            get(list_by_elections).post(declare_by_election),
        )
        .route(
            "/api/v1/governance/by-election/elections/{by_election_id}",
            get(get_by_election),
        )
        .route(
            "/api/v1/governance/by-election/elections/{by_election_id}/status",
            axum::routing::patch(transition_by_election_status),
        )
        .with_state(state)
}
