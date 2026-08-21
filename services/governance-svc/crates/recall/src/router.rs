use crate::handlers::*;
use axum::{
    routing::{get, post, put},
    Router,
};

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        // M48 recall petition endpoints
        .route(
            "/v1/recall/petitions",
            get(list_petitions).post(create_petition),
        )
        .route("/v1/recall/petitions/{id}", get(get_petition))
        .route(
            "/v1/recall/petitions/{id}/signatories/import",
            post(import_signatories),
        )
        .route(
            "/v1/recall/petitions/{id}/signatories",
            get(list_signatories),
        )
        .route(
            "/v1/recall/petitions/{id}/threshold-report",
            get(get_threshold_report),
        )
        .route("/v1/recall/petitions/{id}/close", put(close_petition))
        .route(
            "/v1/recall/petitions/{id}/election",
            post(create_recall_election),
        )
}
