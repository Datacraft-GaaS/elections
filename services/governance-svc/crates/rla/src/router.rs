use crate::handlers::*;
use axum::{
    routing::{get, patch, post},
    Router,
};

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/rla/audits", get(list_audits).post(create_audit))
        .route("/rla/audits/{run_id}", get(get_audit))
        .route("/rla/audits/{run_id}/samples", get(list_samples))
        .route("/rla/audits/{run_id}/samples/{ballot_id}", get(get_sample))
        .route(
            "/rla/audits/{run_id}/samples/{ballot_id}/count",
            patch(record_count),
        )
        .route("/rla/audits/{run_id}/report", get(get_report))
        .route("/rla/audits/{run_id}/finalise", post(finalise_audit))
}
