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
            "/legal-audit/transmissions",
            get(list_transmissions).post(record_transmission),
        )
        .route(
            "/legal-audit/transmissions/{record_id}/verdict",
            get(get_verdict),
        )
        .route("/legal-audit/verdicts", get(list_verdicts))
        .route(
            "/legal-audit/parties/{party_id}/summary",
            get(party_summary),
        )
        .route("/legal-audit/reports", post(generate_report))
    // Public contract endpoints (governance-legal-audit/audit.yaml)
        .route(
            "/api/v1/governance/legal-audit/transmissions",
            get(list_transmission_legal_records),
        )
        .route(
            "/api/v1/governance/legal-audit/transmissions/{form_id}",
            get(get_transmission_legal_basis),
        )
        .route(
            "/api/v1/governance/legal-audit/certifications",
            post(certify_transmission),
        )
        .route(
            "/api/v1/governance/legal-audit/reports/{report_id}",
            get(get_legal_audit_report),
        )
        .with_state(state)
}