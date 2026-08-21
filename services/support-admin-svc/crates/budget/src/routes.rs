use axum::{
    routing::{get, post, put},
    Router,
};

use crate::handlers;

/// Assembles the full router for the budget service.
pub fn router() -> Router {
    let v1 = Router::new()
        // Budget management
        .route("/v1/budgets", post(handlers::create_budget))
        .route("/v1/budgets/{id}/approve", put(handlers::approve_budget))
        // Procurement
        .route("/v1/procurement", post(handlers::create_procurement))
        // Vouchers
        .route("/v1/vouchers", post(handlers::create_voucher))
        .route("/v1/vouchers/{id}/approve", put(handlers::approve_voucher))
        .route(
            "/v1/vouchers/{id}/counter-approve",
            put(handlers::counter_approve_voucher),
        )
        .route("/v1/vouchers/{id}/pay", put(handlers::pay_voucher))
        // CoB reports
        .route("/v1/reports/cob", get(handlers::get_cob_report))
        .route("/v1/reports/cob/export", get(handlers::export_cob_report));

    let openapi = Router::new()
        // expenditure.yaml operationIds
        .route(
            "/api/v1/support/budget/line-items",
            post(handlers::create_budget_line_item).get(handlers::list_budget_line_items),
        )
        .route(
            "/api/v1/support/budget/expenditures",
            post(handlers::record_expenditure),
        )
        .route(
            "/api/v1/support/budget/reports/summary",
            get(handlers::get_budget_summary),
        )
        .route(
            "/api/v1/support/budget/reports/oag-export",
            get(handlers::get_oag_export),
        );

    Router::new().merge(v1).merge(openapi)
}
