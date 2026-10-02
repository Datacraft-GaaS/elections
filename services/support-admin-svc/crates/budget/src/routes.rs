// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
