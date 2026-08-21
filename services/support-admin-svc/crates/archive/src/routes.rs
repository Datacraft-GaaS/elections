use axum::{
    routing::{get, post, put},
    Router,
};

use crate::handlers;

/// Bind all archive routes and return the combined Router.
pub fn router() -> Router {
    // Public API routes (openapi-contracts/support-archive/archive.yaml)
    let public = Router::new()
        .route(
            "/api/v1/support/archive/collections",
            post(handlers::create_archival_collection).get(handlers::list_archival_collections),
        )
        .route(
            "/api/v1/support/archive/collections/:collection_id",
            get(handlers::get_archival_collection),
        )
        .route(
            "/api/v1/support/archive/retrieval-requests",
            post(handlers::create_retrieval_request),
        )
        .route(
            "/api/v1/support/archive/retrieval-requests/:request_id",
            get(handlers::get_retrieval_request),
        );

    // Module-level routes
    let module = Router::new()
        .route(
            "/v1/archive/collections/:id/ingest",
            post(handlers::start_ingest),
        )
        .route(
            "/v1/archive/collections/:id/ingest/:job_id",
            get(handlers::get_ingest_job),
        )
        .route(
            "/v1/archive/collections/:id/seal",
            put(handlers::seal_collection),
        )
        .route(
            "/v1/archive/collections/:id/attest",
            post(handlers::attest_collection),
        )
        .route(
            "/v1/archive/collections/:id/access",
            post(handlers::request_access),
        )
        .route(
            "/v1/archive/collections/:id/destroy",
            post(handlers::destroy_collection),
        )
        .route(
            "/v1/archive/collections/:id/format-migration",
            get(handlers::list_format_migrations).post(handlers::start_format_migration),
        );

    Router::new().merge(public).merge(module)
}
