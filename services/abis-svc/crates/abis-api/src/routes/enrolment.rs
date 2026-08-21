//! Enrolment job routes.

use std::sync::Arc;

use axum::{
	extract::{Path, State},
	http::StatusCode,
	Json,
};
use serde::{Deserialize, Serialize};
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
	pub created_at: String,
	pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QualityReport {
	pub job_id: String,
	pub fingerprint_nfiq2: Option<f32>,
	pub face_quality: Option<f32>,
	pub iris_quality: Option<f32>,
	pub overall_pass: bool,
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
	State(_state): State<Arc<AppState>>,
	Json(_req): Json<EnrolmentJobRequest>,
) -> (StatusCode, Json<EnrolmentJobAccepted>) {
	let job_id = uuid::Uuid::new_v4().to_string();
	(
		StatusCode::ACCEPTED,
		Json(EnrolmentJobAccepted {
			job_id,
			status: "queued".into(),
			estimated_completion_ms: 2000,
		}),
	)
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
	State(_state): State<Arc<AppState>>,
	Path(id): Path<String>,
) -> (StatusCode, Json<EnrolmentJobStatus>) {
	(
		StatusCode::OK,
		Json(EnrolmentJobStatus {
			job_id: id.clone(),
			subject_id: "subject-001".into(),
			status: "processing".into(),
			modalities_completed: vec!["fingerprint".into()],
			created_at: chrono::Utc::now().to_rfc3339(),
			updated_at: chrono::Utc::now().to_rfc3339(),
		}),
	)
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
	State(_state): State<Arc<AppState>>,
	Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
	(
		StatusCode::OK,
		Json(serde_json::json!({"job_id": id, "status": "withdrawn"})),
	)
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
	State(_state): State<Arc<AppState>>,
	Path(id): Path<String>,
) -> (StatusCode, Json<QualityReport>) {
	(
		StatusCode::OK,
		Json(QualityReport {
			job_id: id,
			fingerprint_nfiq2: Some(60.0),
			face_quality: Some(0.92),
			iris_quality: Some(0.88),
			overall_pass: true,
		}),
	)
}
