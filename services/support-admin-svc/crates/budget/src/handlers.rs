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

use axum::{Json, extract::Path};
use serde_json::{Value, json};
use uuid::Uuid;


// ---------------------------------------------------------------------------
// Budget handlers
// ---------------------------------------------------------------------------

/// POST /v1/budgets
pub async fn create_budget() -> Json<Value> {
    let now = chrono::Utc::now();
    Json(json!({
        "id": Uuid::now_v7(),
        "election_id": Uuid::nil(),
        "financial_year": "2024/2025",
        "total_kes": 0,
        "approved_by": null,
        "status": "draft",
        "created_at": now,
        "updated_at": now,
    }))
}

/// PUT /v1/budgets/{id}/approve
pub async fn approve_budget(Path(id): Path<Uuid>) -> Json<Value> {
    let now = chrono::Utc::now();
    Json(json!({
        "id": id,
        "status": "approved",
        "approved_by": Uuid::now_v7(),
        "updated_at": now,
    }))
}

// ---------------------------------------------------------------------------
// Budget line item handlers (expenditure.yaml operationIds)
// ---------------------------------------------------------------------------

/// POST /api/v1/support/budget/line-items  (operationId: createBudgetLineItem)
pub async fn create_budget_line_item() -> (axum::http::StatusCode, Json<Value>) {
    let now = chrono::Utc::now();
    let body = json!({
        "id": Uuid::now_v7(),
        "budget_id": Uuid::nil(),
        "programme_code": "PROG-001",
        "description": "stub",
        "allocated_kes": 0,
        "committed_kes": 0,
        "spent_kes": 0,
        "created_at": now,
        "updated_at": now,
    });
    (axum::http::StatusCode::CREATED, Json(body))
}

/// GET /api/v1/support/budget/line-items  (operationId: listBudgetLineItems)
pub async fn list_budget_line_items() -> Json<Value> {
    Json(json!({ "items": [], "total": 0 }))
}

// ---------------------------------------------------------------------------
// Expenditure handlers
// ---------------------------------------------------------------------------

/// POST /api/v1/support/budget/expenditures  (operationId: recordExpenditure)
pub async fn record_expenditure() -> (axum::http::StatusCode, Json<Value>) {
    let now = chrono::Utc::now();
    let body = json!({
        "id": Uuid::now_v7(),
        "budget_line_id": Uuid::nil(),
        "vendor_id": Uuid::nil(),
        "ppra_ref": "PPRA-0000",
        "amount_kes": 0,
        "procurement_method": "open_tender",
        "status": "pending",
        "created_at": now,
        "updated_at": now,
    });
    (axum::http::StatusCode::CREATED, Json(body))
}

// ---------------------------------------------------------------------------
// Procurement handlers
// ---------------------------------------------------------------------------

/// POST /v1/procurement
pub async fn create_procurement() -> (axum::http::StatusCode, Json<Value>) {
    let now = chrono::Utc::now();
    let body = json!({
        "id": Uuid::now_v7(),
        "budget_line_id": Uuid::nil(),
        "vendor_id": Uuid::nil(),
        "ppra_ref": "PPRA-0000",
        "amount_kes": 0,
        "procurement_method": "open_tender",
        "status": "pending",
        "created_at": now,
        "updated_at": now,
    });
    (axum::http::StatusCode::CREATED, Json(body))
}

// ---------------------------------------------------------------------------
// Voucher handlers
// ---------------------------------------------------------------------------

/// POST /v1/vouchers
pub async fn create_voucher() -> (axum::http::StatusCode, Json<Value>) {
    let now = chrono::Utc::now();
    let body = json!({
        "id": Uuid::now_v7(),
        "procurement_id": Uuid::nil(),
        "amount_kes": 0,
        "initiated_by": Uuid::nil(),
        "approved_by": null,
        "counter_approved_by": null,
        "ifmis_ref": null,
        "status": "pending",
        "created_at": now,
        "updated_at": now,
    });
    (axum::http::StatusCode::CREATED, Json(body))
}

/// PUT /v1/vouchers/{id}/approve
pub async fn approve_voucher(Path(id): Path<Uuid>) -> Json<Value> {
    let now = chrono::Utc::now();
    Json(json!({ "id": id, "status": "approved", "updated_at": now }))
}

/// PUT /v1/vouchers/{id}/counter-approve
pub async fn counter_approve_voucher(Path(id): Path<Uuid>) -> Json<Value> {
    let now = chrono::Utc::now();
    Json(json!({ "id": id, "status": "counter_approved", "updated_at": now }))
}

/// PUT /v1/vouchers/{id}/pay
pub async fn pay_voucher(Path(id): Path<Uuid>) -> Json<Value> {
    let now = chrono::Utc::now();
    Json(json!({ "id": id, "status": "paid", "ifmis_ref": "IFMIS-0000", "updated_at": now }))
}

// ---------------------------------------------------------------------------
// Report handlers
// ---------------------------------------------------------------------------

/// GET /api/v1/support/budget/reports/summary  (operationId: getBudgetSummary)
pub async fn get_budget_summary() -> Json<Value> {
    Json(json!({
        "total_allocated": 0,
        "total_committed": 0,
        "total_spent": 0,
        "variance": 0,
    }))
}

/// GET /api/v1/support/budget/reports/oag-export  (operationId: getOagExport)
pub async fn get_oag_export() -> Json<Value> {
    Json(json!({
        "packages": [],
        "generated_at": chrono::Utc::now(),
    }))
}

/// GET /v1/reports/cob
pub async fn get_cob_report() -> Json<Value> {
    Json(json!({
        "reports": [],
        "total": 0,
    }))
}

/// GET /v1/reports/cob/export
pub async fn export_cob_report() -> Json<Value> {
    Json(json!({
        "export_url": null,
        "generated_at": chrono::Utc::now(),
    }))
}
