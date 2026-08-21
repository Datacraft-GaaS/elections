use crate::handlers::*;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build_router() -> Router {
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
}
