use crate::handlers::*;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build_router() -> Router {
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
}
