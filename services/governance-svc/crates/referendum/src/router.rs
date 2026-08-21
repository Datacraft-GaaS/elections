use crate::handlers::*;
use axum::{
    routing::{get, post, put},
    Router,
};

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        // M49 referendum endpoints
        .route(
            "/v1/referendums",
            get(list_referendums).post(create_referendum),
        )
        .route("/v1/referendums/{id}", get(get_referendum))
        .route("/v1/referendums/{id}/activate", put(activate_referendum))
        .route(
            "/v1/referendums/{id}/county-results",
            post(submit_county_result),
        )
        .route(
            "/v1/referendums/{id}/county-results/{county}/certify",
            put(certify_county_result),
        )
        .route(
            "/v1/referendums/{id}/national-result",
            get(get_national_result),
        )
        .route("/v1/referendums/{id}/declare", post(declare_referendum))
}
