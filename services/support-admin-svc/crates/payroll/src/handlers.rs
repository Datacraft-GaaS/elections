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

//! Stub HTTP handlers for the payroll service (WP-5.6).
//!
//! All handlers return 200/201 with an empty JSON body. Real implementations
//! will persist via sqlx and invoke domain validation.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    domain::{
        self, ApplicationStatus, DocumentRequirement, DocumentSubmission, DomainError,
        JobApplication, Verdict, DOCUMENT_CHANNELS, DUAL_CONTROL_DOC_TYPE,
    },
    error::ApiError,
    state::AppState,
};

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
    (
        StatusCode::CREATED,
        Json(json!({ "job_id": job_id, "status": "queued" })),
    )
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
// HR: applications (portal intake)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct NewDocument {
    pub doc_type: String,
    pub content_hash_hex: String,
    pub object_key: String,
    pub channel: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateApplicationRequest {
    pub applicant_name: String,
    /// Raw national ID; hashed server-side, never persisted in the clear.
    pub national_id: String,
    pub role_code: String,
    pub county: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub documents: Vec<NewDocument>,
}

/// POST /v1/hr/applications — public portal intake (design §1/§2).
///
/// Idempotent on the national-ID hash: re-submitting the same ID returns the
/// original application (200) rather than a duplicate.
pub async fn create_application(
    State(state): State<AppState>,
    Json(req): Json<CreateApplicationRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let name = req.applicant_name.trim();
    let national_id = req.national_id.trim();
    let role_code = req.role_code.trim();
    let county = req.county.trim();
    if name.is_empty() || national_id.is_empty() || role_code.is_empty() || county.is_empty() {
        return Err(ApiError::BadRequest(
            "applicant_name, national_id, role_code and county are required".into(),
        ));
    }

    let national_id_hash = domain::hash_sha256_hex(national_id);

    if let Some(existing) = state.applications.find_by_national_id(&national_id_hash) {
        let body = serde_json::to_value(&existing)
            .map_err(|e| ApiError::Internal(anyhow::Error::new(e)))?;
        return Ok((
            StatusCode::OK,
            Json(json!({ "application": body, "duplicate": true })),
        ));
    }

    let application = JobApplication {
        id: Uuid::now_v7(),
        application_ref: domain::generate_application_ref(),
        role_code: role_code.to_string(),
        county: county.to_string(),
        applicant_name: name.to_string(),
        national_id_hash,
        contact_phone_hash: req
            .phone
            .as_deref()
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(domain::hash_sha256_hex),
        contact_email_hash: req
            .email
            .as_deref()
            .map(str::trim)
            .filter(|e| !e.is_empty())
            .map(domain::hash_sha256_hex),
        status: ApplicationStatus::Applied,
        applied_at: Utc::now(),
        training_gate_passed: false,
    };
    let id = application.id;

    for doc in &req.documents {
        if !domain::is_valid_content_hash(&doc.content_hash_hex) {
            return Err(ApiError::Domain(DomainError::InvalidContentHash));
        }
        if !DOCUMENT_CHANNELS.contains(&doc.channel.as_str()) {
            return Err(ApiError::Domain(DomainError::InvalidChannel));
        }
        if state
            .documents
            .find_idempotent(id, &doc.doc_type, &doc.content_hash_hex)
            .is_none()
        {
            let submission = DocumentSubmission {
                id: Uuid::now_v7(),
                application_id: id,
                doc_type: doc.doc_type.clone(),
                content_hash_hex: doc.content_hash_hex.clone(),
                object_key: doc.object_key.clone(),
                channel: doc.channel.clone(),
                submitted_at: Utc::now(),
                superseded_by: None,
                verdict: Verdict::Pending,
            };
            state.documents.insert(submission);
        }
    }

    state.applications.insert(application.clone());
    let body = serde_json::to_value(&application)
        .map_err(|e| ApiError::Internal(anyhow::Error::new(e)))?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "application": body, "duplicate": false })),
    ))
}

/// GET /v1/hr/applications/{id}
pub async fn get_application(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let application = state.applications.get(id).ok_or(ApiError::NotFound)?;
    Ok(Json(
        serde_json::to_value(application).map_err(|e| ApiError::Internal(anyhow::Error::new(e)))?,
    ))
}

/// GET /v1/hr/applications/by-ref/{reference} — portal status lookup.
pub async fn get_application_by_reference(
    State(state): State<AppState>,
    Path(reference): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let application = state
        .applications
        .find_by_ref(reference.trim())
        .ok_or(ApiError::NotFound)?;
    Ok(Json(
        serde_json::to_value(application).map_err(|e| ApiError::Internal(anyhow::Error::new(e)))?,
    ))
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
    (
        StatusCode::CREATED,
        Json(json!({ "id": id, "status": "draft" })),
    )
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

// ---------------------------------------------------------------------------
// HR: documents (workforce-lifecycle §2)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListRequirementsQuery {
    pub role_code: Option<String>,
}

/// GET /v1/hr/documents/requirements?role_code=
pub async fn list_document_requirements(Query(query): Query<ListRequirementsQuery>) -> Json<Value> {
    let items: Vec<DocumentRequirement> = match &query.role_code {
        Some(role_code) => domain::seed_requirements(role_code),
        None => domain::seed_all_requirements(),
    };
    Json(json!({ "items": items }))
}

#[derive(Debug, Deserialize)]
pub struct SubmitDocumentRequest {
    pub application_id: Uuid,
    pub doc_type: String,
    pub content_hash_hex: String,
    pub object_key: String,
    pub channel: String,
}

/// POST /v1/hr/documents/submissions — idempotent per
/// `(application_id, doc_type, content_hash)`; replays return 200.
pub async fn submit_document(
    State(state): State<AppState>,
    Json(req): Json<SubmitDocumentRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    if !domain::is_valid_content_hash(&req.content_hash_hex) {
        return Err(ApiError::Domain(DomainError::InvalidContentHash));
    }
    if !DOCUMENT_CHANNELS.contains(&req.channel.as_str()) {
        return Err(ApiError::Domain(DomainError::InvalidChannel));
    }
    if let Some(existing) =
        state
            .documents
            .find_idempotent(req.application_id, &req.doc_type, &req.content_hash_hex)
    {
        return Ok((
            StatusCode::OK,
            Json(json!({ "id": existing, "duplicate": true })),
        ));
    }
    let submission = DocumentSubmission {
        id: Uuid::now_v7(),
        application_id: req.application_id,
        doc_type: req.doc_type,
        content_hash_hex: req.content_hash_hex,
        object_key: req.object_key,
        channel: req.channel,
        submitted_at: Utc::now(),
        superseded_by: None,
        verdict: Verdict::Pending,
    };
    let id = submission.id;
    state.documents.insert(submission);
    Ok((
        StatusCode::CREATED,
        Json(json!({ "id": id, "duplicate": false })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct SupersedeDocumentRequest {
    pub content_hash_hex: String,
    pub object_key: String,
}

/// POST /v1/hr/documents/{id}/supersede — retains history by marking the old
/// submission `superseded_by` the new pending one.
pub async fn supersede_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<SupersedeDocumentRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    if !domain::is_valid_content_hash(&req.content_hash_hex) {
        return Err(ApiError::Domain(DomainError::InvalidContentHash));
    }
    let old = state.documents.get(id).ok_or(ApiError::NotFound)?;
    if old.superseded_by.is_some() {
        return Err(ApiError::Conflict("submission already superseded".into()));
    }
    let replacement = DocumentSubmission {
        id: Uuid::now_v7(),
        application_id: old.application_id,
        doc_type: old.doc_type.clone(),
        content_hash_hex: req.content_hash_hex,
        object_key: req.object_key,
        channel: old.channel,
        submitted_at: Utc::now(),
        superseded_by: None,
        verdict: Verdict::Pending,
    };
    let new_id = replacement.id;
    state.documents.insert(replacement);
    assert!(state.documents.mark_superseded(id, new_id));
    Ok((
        StatusCode::CREATED,
        Json(json!({ "id": new_id, "supersedes": id })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct VerdictRequest {
    /// "verified" | "rejected".
    pub verdict: String,
    pub verifier_id: Uuid,
    pub reason: Option<String>,
}

/// POST /v1/hr/documents/{id}/verdict — rejecting national_id requires an
/// `X-Second-Reviewer` header distinct from `verifier_id` (dual control).
pub async fn record_verdict(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<VerdictRequest>,
) -> Result<Json<Value>, ApiError> {
    let verdict_json = apply_verdict(
        &state,
        &headers,
        id,
        &req.verdict,
        req.verifier_id,
        req.reason,
    )?;
    Ok(Json(json!({ "id": id, "verdict": verdict_json })))
}

/// Shared verdict application used by single and bulk endpoints.
fn apply_verdict(
    state: &AppState,
    headers: &HeaderMap,
    id: Uuid,
    verdict: &str,
    verifier_id: Uuid,
    reason: Option<String>,
) -> Result<Value, ApiError> {
    let submission = state.documents.get(id).ok_or(ApiError::NotFound)?;
    if submission.verdict != Verdict::Pending {
        return Err(ApiError::Conflict("verdict already recorded".into()));
    }
    let verdict = match verdict {
        "verified" => Verdict::Verified {
            verifier_id,
            verified_at: Utc::now(),
        },
        "rejected" => {
            let reason = reason
                .filter(|reason| !reason.trim().is_empty())
                .ok_or(DomainError::RejectionReasonRequired)?;
            if submission.doc_type == DUAL_CONTROL_DOC_TYPE {
                headers
                    .get("x-second-reviewer")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| Uuid::parse_str(value).ok())
                    .filter(|second| *second != verifier_id)
                    .ok_or(DomainError::DualControlViolation)?;
            }
            Verdict::Rejected {
                verifier_id,
                reason,
                verified_at: Utc::now(),
            }
        }
        other => {
            return Err(ApiError::BadRequest(format!(
                "invalid verdict: {other} (expected verified|rejected)"
            )))
        }
    };
    if !state.documents.apply_verdict(id, verdict.clone()) {
        return Err(ApiError::Conflict("verdict already recorded".into()));
    }
    serde_json::to_value(&verdict).map_err(|e| ApiError::Internal(anyhow::Error::new(e)))
}

#[derive(Debug, Deserialize)]
pub struct BulkVerdictItem {
    pub id: Uuid,
    pub verdict: String,
    pub verifier_id: Uuid,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkVerdictRequest {
    pub verdicts: Vec<BulkVerdictItem>,
}

/// POST /v1/hr/documents/bulk-verdict — apply verdicts to many submissions.
/// Dual control still applies per national_id rejection; per-item failures are
/// collected rather than fatal.
pub async fn bulk_record_verdict(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<BulkVerdictRequest>,
) -> Result<Json<Value>, ApiError> {
    let mut results = Vec::new();
    for item in req.verdicts {
        match apply_verdict(
            &state,
            &headers,
            item.id,
            &item.verdict,
            item.verifier_id,
            item.reason,
        ) {
            Ok(verdict_json) => results.push(json!({ "id": item.id, "verdict": verdict_json })),
            Err(e) => results.push(json!({ "id": item.id, "error": e.to_string() })),
        }
    }
    Ok(Json(json!({ "results": results })))
}

#[derive(Debug, Deserialize)]
pub struct CompletenessQuery {
    /// Optional role scoping; defaults to the full mandatory matrix union.
    pub role_code: Option<String>,
}

/// GET /v1/hr/applications/{id}/documents/completeness — probe used by the
/// recruitment state machine.
pub async fn get_application_completeness(
    State(state): State<AppState>,
    Path(application_id): Path<Uuid>,
    Query(query): Query<CompletenessQuery>,
) -> Result<Json<Value>, ApiError> {
    let requirements = match &query.role_code {
        Some(role_code) => domain::seed_requirements(role_code),
        None => domain::seed_all_requirements(),
    };
    let submissions = state.documents.for_application(application_id);
    let completeness = domain::completeness(&requirements, &submissions, Utc::now());
    Ok(Json(
        serde_json::to_value(completeness)
            .map_err(|e| ApiError::Internal(anyhow::Error::new(e)))?,
    ))
}

#[derive(Debug, Deserialize)]
pub struct ExpirySweepQuery {
    /// RFC 3339 instant; defaults to now.
    pub as_of: Option<String>,
}

/// POST /v1/hr/documents/expiry-sweep — lists application ids whose
/// completeness regressed due to expiry as of `as_of`.
pub async fn expiry_sweep(
    State(state): State<AppState>,
    Query(query): Query<ExpirySweepQuery>,
) -> Result<Json<Value>, ApiError> {
    let as_of = match &query.as_of {
        Some(raw) => DateTime::parse_from_rfc3339(raw)
            .map_err(|_| ApiError::BadRequest(format!("invalid as_of: {raw}")))?
            .with_timezone(&Utc),
        None => Utc::now(),
    };
    let submissions = state.documents.all();
    let regressed =
        domain::expiry_regressions(&domain::seed_all_requirements(), &submissions, as_of);
    Ok(Json(json!({
        "as_of": as_of,
        "regressed_application_ids": regressed,
    })))
}
