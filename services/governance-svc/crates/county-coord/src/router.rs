use crate::handlers::*;
use crate::state::SharedState;
use axum::{
    routing::{get, post, put},
    Router,
};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/coordination/mous", get(list_mous).post(create_mou))
        .route("/v1/coordination/mous/{id}/sign", put(sign_mou))
        .route(
            "/v1/coordination/datasets",
            get(list_datasets).post(submit_dataset),
        )
        .route(
            "/v1/coordination/incidents",
            get(list_incidents).post(create_incident),
        )
        .route(
            "/v1/coordination/incidents/{id}/resolve",
            put(resolve_incident),
        )
        .route("/v1/auth/saml/acs", post(saml_acs))
    // Public contract endpoints (governance-county-coord/coordination.yaml)
        .route(
            "/api/v1/governance/county-coord/status-reports",
            get(list_county_status_reports).post(submit_county_status_report),
        )
        .route(
            "/api/v1/governance/county-coord/escalations",
            get(list_escalations).post(escalate_to_national),
        )
        .with_state(state)
}
