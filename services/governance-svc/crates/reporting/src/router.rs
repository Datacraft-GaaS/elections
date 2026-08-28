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
            "/reporting/reports",
            get(list_reports).post(generate_report),
        )
        .route("/reporting/reports/{report_id}", get(get_report))
        .route(
            "/reporting/testimony-workspaces",
            get(list_testimony_workspaces).post(create_testimony_workspace),
        )
        .route(
            "/reporting/judiciary-handovers",
            post(record_judiciary_handover),
        )
        .route("/reporting/sc-preregistrations", post(register_sc_it))
    // Public contract endpoints (governance-reporting/reports.yaml)
        .route(
            "/api/v1/governance/reporting/parliamentary-reports",
            get(list_parliamentary_reports).post(generate_parliamentary_report),
        )
        .route(
            "/api/v1/governance/reporting/parliamentary-reports/{report_id}",
            get(get_parliamentary_report),
        )
        .route(
            "/api/v1/governance/reporting/oag-submissions",
            get(list_oag_submissions).post(submit_to_oag),
        )
        .with_state(state)
}
