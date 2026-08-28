use crate::handlers::*;
use crate::spec_state::SharedSpecState;
use crate::store::Store;
use crate::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build_router(store: Store, spec_state: SharedSpecState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/rla/audits", get(list_audits).post(create_audit))
        .route("/rla/audits/{run_id}", get(get_audit))
        .route("/rla/audits/{run_id}/samples", get(list_samples))
        // ballot_id embeds the draw's "batch/position" form, so it may carry a slash.
        .route(
            "/rla/audits/{run_id}/samples/{*ballot_id}",
            get(get_sample).post(record_interpretation),
        )
        .route("/rla/audits/{run_id}/report", get(get_report))
        .route("/rla/audits/{run_id}/finalise", post(finalise_audit))
        .route("/rla/audits/{run_id}/next-block", get(next_block))
        // Public contract endpoints (governance-rla/audit.yaml)
        .route(
            "/api/v1/governance/rla/audits",
            get(list_rla_audits).post(initiate_rla_audit),
        )
        .route(
            "/api/v1/governance/rla/audits/{audit_id}",
            get(get_rla_audit),
        )
        .route(
            "/api/v1/governance/rla/audits/{audit_id}/samples",
            post(draw_audit_sample),
        )
        .route(
            "/api/v1/governance/rla/audits/{audit_id}/counts",
            post(submit_manual_count),
        )
        .route(
            "/api/v1/governance/rla/audits/{audit_id}/results",
            get(get_rla_audit_results),
        )
        .with_state(AppState { store, spec_state })
}
