//! Enrolment job routes.

use std::sync::Arc;

use axum::{
	response::IntoResponse,
	extract::{Path, State},
	http::StatusCode,
	Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EnrolmentJobRequest {
	pub subject_id: String,
	pub modalities: Vec<String>,
	pub demographics: Option<serde_json::Value>,
	pub operator_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EnrolmentJobAccepted {
	pub job_id: String,
	pub status: String,
	pub estimated_completion_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EnrolmentJobStatus {
	pub job_id: String,
	pub subject_id: String,
	pub status: String,
	pub modalities_completed: Vec<String>,
	/// What the kit was asked to capture, so a partial capture is visible.
	pub requested_modalities: Vec<String>,
	pub created_at: String,
	pub updated_at: String,
	/// Why the quality figures are not available, when they are not.
	pub blocking_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QualityReport {
	pub job_id: String,
	pub fingerprint_nfiq2: Option<f32>,
	pub face_quality: Option<f32>,
	pub iris_quality: Option<f32>,
	pub overall_pass: bool,
}

/// Record a captured modality against a job.
///
/// The kit reports what it captured; this is how a job progresses past `queued` and
/// becomes `captured`, which is what makes a quality report reachable at all.
#[derive(Debug, Deserialize)]
pub struct RecordModalityRequest {
	pub modality: String,
	pub subject_id_hash: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /v1/enrolment/jobs
#[utoipa::path(
    post,
    path = "/v1/enrolment/jobs",
    tag = "enrolment",
    request_body = EnrolmentJobRequest,
    responses(
        (status = 202, description = "Job accepted", body = EnrolmentJobAccepted),
        (status = 400, description = "Bad request"),
    )
)]
pub async fn create_enrolment_job(
	State(state): State<Arc<AppState>>,
	Json(req): Json<EnrolmentJobRequest>,
) -> axum::response::Response {
	// Previously returned a job id and `status: "queued"` and stored nothing, so
	// the job could never be read back.
	if req.subject_id.trim().is_empty() {
		return (
			StatusCode::UNPROCESSABLE_ENTITY,
			Json(serde_json::json!({"detail": "subject_id is required"})),
		)
			.into_response();
	}
	if req.modalities.is_empty() {
		return (
			StatusCode::UNPROCESSABLE_ENTITY,
			Json(serde_json::json!({"detail": "at least one modality is required"})),
		)
			.into_response();
	}
	let job = state.store.create_job(req.subject_id.trim(), req.modalities.clone());
	// A vendor is registered by a kit from it enrolling a subject, so the
	// enrolment is what makes it appear. It is `unregistered` until then and is
	// never reported healthy on the strength of being listed.
	if let Some(vendor) = req
		.demographics
		.as_ref()
		.and_then(|d| d.get("vendor_id"))
		.and_then(|v| v.as_str())
		.map(str::trim)
		.filter(|v| !v.is_empty())
	{
		state.store.register_vendor(vendor, vendor, job.requested_modalities.clone());
		state.store.note_enrolment(vendor);
	}
	(
		StatusCode::ACCEPTED,
		Json(EnrolmentJobAccepted {
			job_id: job.job_id,
			status: job.status.wire().to_string(),
			estimated_completion_ms: 0,
		}),
	)
		.into_response()
}

/// GET /v1/enrolment/jobs/{id}
#[utoipa::path(
    get,
    path = "/v1/enrolment/jobs/{id}",
    tag = "enrolment",
    params(("id" = String, Path, description = "Job ID")),
    responses(
        (status = 200, description = "Job status", body = EnrolmentJobStatus),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_enrolment_job(
	State(state): State<Arc<AppState>>,
	Path(id): Path<String>,
) -> axum::response::Response {
	// Previously returned `subject_id: "subject-001"`, `status: "processing"` and
	// `modalities_completed: ["fingerprint"]` for **any** id. A fabricated subject
	// identity and a fabricated capture state, for every job in the country.
	match state.store.job(&id) {
		Some(job) => (
			StatusCode::OK,
			Json(EnrolmentJobStatus {
				job_id: job.job_id,
				subject_id: job.subject_id,
				status: job.status.wire().to_string(),
				modalities_completed: job.modalities_completed,
				requested_modalities: job.requested_modalities,
				created_at: job.created_at,
				updated_at: job.updated_at,
				blocking_reason: job.blocking_reason,
			}),
		)
			.into_response(),
		None => (
			StatusCode::NOT_FOUND,
			Json(serde_json::json!({"detail": format!("no enrolment job with id {id}")})),
		)
			.into_response(),
	}
}

/// PATCH /v1/enrolment/jobs/{id}/withdraw
#[utoipa::path(
    patch,
    path = "/v1/enrolment/jobs/{id}/withdraw",
    tag = "enrolment",
    params(("id" = String, Path, description = "Job ID")),
    responses(
        (status = 200, description = "Withdrawn"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn withdraw_enrolment_job(
	State(state): State<Arc<AppState>>,
	Path(id): Path<String>,
) -> axum::response::Response {
	// Previously returned `status: "withdrawn"` for any id.
	match state.store.withdraw(&id) {
		Ok(job) => (
			StatusCode::OK,
			Json(serde_json::json!({
				"job_id": job.job_id,
				"status": job.status.wire(),
				"updated_at": job.updated_at,
			})),
		)
			.into_response(),
		Err(detail) if detail.starts_with("no enrolment job") => {
			(StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": detail}))).into_response()
		}
		Err(detail) => (StatusCode::CONFLICT, Json(serde_json::json!({"detail": detail}))).into_response(),
	}
}

/// GET /v1/enrolment/jobs/{id}/quality-report
#[utoipa::path(
    get,
    path = "/v1/enrolment/jobs/{id}/quality-report",
    tag = "enrolment",
    params(("id" = String, Path, description = "Job ID")),
    responses(
        (status = 200, description = "Quality report", body = QualityReport),
    )
)]
pub async fn get_quality_report(
	State(state): State<Arc<AppState>>,
	Path(id): Path<String>,
) -> axum::response::Response {
	// Previously returned `fingerprint_nfiq2: 60.0`, `face_quality: 0.92`,
	// `iris_quality: 0.88` and **`overall_pass: true`** for any id. A quality report
	// that passes is the record on which an enrolment is allowed to proceed.
	let Some(job) = state.store.job(&id) else {
		return (
			StatusCode::NOT_FOUND,
			Json(serde_json::json!({"detail": format!("no enrolment job with id {id}")})),
		)
			.into_response();
	};
	// The three quality figures come from the kit's sensor and the matcher, and
	// neither is reachable from this process. They are null, and the verdict is
	// not `overall_pass` -- a null is a figure nobody has measured.
	if job.status != crate::store::JobStatus::Captured {
		return (
			StatusCode::CONFLICT,
			Json(serde_json::json!({
				"job_id": id,
				"status": job.status.wire(),
				"detail": format!(
				 "job is {}; capture quality is reported once every requested modality is captured",
				 job.status.wire()
				),
			})),
		)
			.into_response();
	}
	(
		StatusCode::OK,
		Json(serde_json::json!({
			"job_id": id,
			"fingerprint_nfiq2": Value::Null,
			"face_quality": Value::Null,
			"iris_quality": Value::Null,
			"overall_pass": Value::Null,
			"modalities_captured": job.modalities_completed,
			"blocking_reason": "the quality figures come from the kit's sensor and the matcher, \
							 neither of which is reachable from this process; no figure here is \
							 measured, so no pass verdict is issued",
		})),
	)
		.into_response()
}


/// POST /jobs/{id}/modalities — record a modality the kit captured.
pub async fn record_modality(
	State(state): State<Arc<AppState>>,
	Path(id): Path<String>,
	Json(req): Json<RecordModalityRequest>,
) -> axum::response::Response {
	let modality = req.modality.trim();
	if modality.is_empty() {
		return (
			StatusCode::UNPROCESSABLE_ENTITY,
			Json(serde_json::json!({"detail": "modality is required"})),
		)
			.into_response();
	}
	match state.store.record_modality(&id, modality) {
		Ok(job) => (
			StatusCode::OK,
			Json(serde_json::json!({
				"job_id": job.job_id,
				"status": job.status.wire(),
				"modalities_completed": job.modalities_completed,
			})),
		)
			.into_response(),
		Err(detail) if detail.starts_with("no enrolment job") => {
			(StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": detail}))).into_response()
		}
		Err(detail) if detail.contains("was not requested") => {
			(StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({"detail": detail}))).into_response()
		}
		Err(detail) => (StatusCode::CONFLICT, Json(serde_json::json!({"detail": detail}))).into_response(),
	}
}
