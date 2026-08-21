//! Stub HTTP handlers for the payroll service (WP-5.6).
//!
//! All handlers return 200/201 with an empty JSON body. Real implementations
//! will persist via sqlx and invoke domain validation.

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Personnel (openapi-contracts: support-payroll/payroll.yaml)
// ---------------------------------------------------------------------------

/// operationId: registerPersonnel
pub async fn register_personnel(Json(_body): Json<Value>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "status": "accepted" })))
}

/// operationId: listPersonnel
pub async fn list_personnel() -> impl IntoResponse {
    Json(json!({ "items": [], "total": 0 }))
}

// ---------------------------------------------------------------------------
// Disbursements
// ---------------------------------------------------------------------------

/// operationId: initiateDisbursement
pub async fn initiate_disbursement(Json(_body): Json<Value>) -> impl IntoResponse {
    let job_id = Uuid::now_v7();
    (StatusCode::CREATED, Json(json!({ "job_id": job_id, "status": "queued" })))
}

/// operationId: getDisbursementJob
pub async fn get_disbursement_job(Path(job_id): Path<Uuid>) -> impl IntoResponse {
    Json(json!({ "job_id": job_id, "status": "pending" }))
}

// ---------------------------------------------------------------------------
// Reports
// ---------------------------------------------------------------------------

/// operationId: getCountyPayrollReport
pub async fn get_county_payroll_report(Path(county_code): Path<String>) -> impl IntoResponse {
    Json(json!({ "county_code": county_code, "lines": [] }))
}

// ---------------------------------------------------------------------------
// HR: applications
// ---------------------------------------------------------------------------

pub async fn create_application(Json(_body): Json<Value>) -> impl IntoResponse {
    let id = Uuid::now_v7();
    (StatusCode::CREATED, Json(json!({ "id": id })))
}

pub async fn get_application(Path(id): Path<Uuid>) -> impl IntoResponse {
    Json(json!({ "id": id }))
}

// ---------------------------------------------------------------------------
// HR: contracts
// ---------------------------------------------------------------------------

pub async fn create_contract(Json(_body): Json<Value>) -> impl IntoResponse {
    let id = Uuid::now_v7();
    (StatusCode::CREATED, Json(json!({ "id": id })))
}

pub async fn terminate_contract(Path(id): Path<Uuid>) -> impl IntoResponse {
    Json(json!({ "id": id, "status": "terminated" }))
}

// ---------------------------------------------------------------------------
// Payroll runs
// ---------------------------------------------------------------------------

pub async fn create_payroll_run(Json(_body): Json<Value>) -> impl IntoResponse {
    let id = Uuid::now_v7();
    (StatusCode::CREATED, Json(json!({ "id": id, "status": "draft" })))
}

pub async fn approve_payroll_run(Path(id): Path<Uuid>) -> impl IntoResponse {
    Json(json!({ "id": id, "status": "approved" }))
}

pub async fn disburse_payroll_run(Path(id): Path<Uuid>) -> impl IntoResponse {
    Json(json!({ "id": id, "status": "paid" }))
}

pub async fn get_payroll_report(Path(id): Path<Uuid>) -> impl IntoResponse {
    Json(json!({ "payroll_run_id": id, "lines": [] }))
}

pub async fn export_payroll_run(Path(id): Path<Uuid>) -> impl IntoResponse {
    Json(json!({ "payroll_run_id": id, "export_url": null }))
}

// ---------------------------------------------------------------------------
// Payslips
// ---------------------------------------------------------------------------

pub async fn get_payslip(Path((staff_id, period)): Path<(Uuid, String)>) -> impl IntoResponse {
    Json(json!({ "staff_id": staff_id, "period": period, "lines": [] }))
}
