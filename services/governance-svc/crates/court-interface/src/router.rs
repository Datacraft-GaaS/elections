use crate::handlers::*;
use crate::state::SharedState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/court/petitions", get(list_petitions).post(file_petition))
        .route("/court/petitions/{petition_id}", get(get_petition))
        .route(
            "/court/petitions/{petition_id}/evidence/request",
            post(request_evidence),
        )
        .route(
            "/court/petitions/{petition_id}/evidence/bundle",
            get(get_evidence_bundle),
        )
        .route(
            "/court/petitions/{petition_id}/witnesses",
            get(list_witnesses).post(register_witness),
        )
        .route(
            "/court/petitions/{petition_id}/supplementary",
            post(add_supplementary_request),
        )
    // Public contract endpoints (governance-court/interface.yaml)
        .route(
            "/api/v1/governance/court/petitions",
            get(list_petitions).post(file_petition),
        )
        .route(
            "/api/v1/governance/court/petitions/{petition_id}",
            get(get_petition),
        )
        .route(
            "/api/v1/governance/court/petitions/{petition_id}/evidence-requests",
            post(create_evidence_request),
        )
        .route(
            "/api/v1/governance/court/petitions/{petition_id}/judgments",
            post(record_judgment),
        )
        .with_state(state)
}
