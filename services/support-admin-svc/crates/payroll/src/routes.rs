//! Axum router for the payroll service (WP-5.6).

use axum::{
    routing::{get, post, put},
    Router,
};

use crate::{handlers, state::AppState};

pub fn router() -> Router {
    Router::new()
        // openapi-contracts: support-payroll/payroll.yaml
        .route(
            "/api/v1/support/payroll/personnel",
            post(handlers::register_personnel).get(handlers::list_personnel),
        )
        .route(
            "/api/v1/support/payroll/disbursements",
            post(handlers::initiate_disbursement),
        )
        .route(
            "/api/v1/support/payroll/disbursements/{job_id}",
            get(handlers::get_disbursement_job),
        )
        .route(
            "/api/v1/support/payroll/reports/county/{county_code}",
            get(handlers::get_county_payroll_report),
        )
        // HR applications
        .route("/v1/hr/applications", post(handlers::create_application))
        .route("/v1/hr/applications/{id}", get(handlers::get_application))
        .route(
            "/v1/hr/applications/by-ref/{reference}",
            get(handlers::get_application_by_reference),
        )
        // HR contracts
        .route("/v1/hr/contracts", post(handlers::create_contract))
        .route(
            "/v1/hr/contracts/{id}/terminate",
            put(handlers::terminate_contract),
        )
        // Payroll runs
        .route("/v1/payroll/runs", post(handlers::create_payroll_run))
        .route(
            "/v1/payroll/runs/{id}/approve",
            put(handlers::approve_payroll_run),
        )
        .route(
            "/v1/payroll/runs/{id}/disburse",
            put(handlers::disburse_payroll_run),
        )
        .route(
            "/v1/payroll/runs/{id}/report",
            get(handlers::get_payroll_report),
        )
        .route(
            "/v1/payroll/runs/{id}/export",
            get(handlers::export_payroll_run),
        )
        // Payslips
        .route(
            "/v1/payroll/payslips/{staff_id}/{period}",
            get(handlers::get_payslip),
        )
        // HR documents (workforce-lifecycle §2)
        .route(
            "/v1/hr/documents/requirements",
            get(handlers::list_document_requirements),
        )
        .route(
            "/v1/hr/documents/submissions",
            post(handlers::submit_document),
        )
        .route(
            "/v1/hr/documents/{id}/supersede",
            post(handlers::supersede_document),
        )
        .route(
            "/v1/hr/documents/{id}/verdict",
            post(handlers::record_verdict),
        )
        .route(
            "/v1/hr/documents/bulk-verdict",
            post(handlers::bulk_record_verdict),
        )
        .route(
            "/v1/hr/documents/expiry-sweep",
            post(handlers::expiry_sweep),
        )
        .route(
            "/v1/hr/applications/{id}/documents/completeness",
            get(handlers::get_application_completeness),
        )
        .with_state(AppState::default())
}
