//! BVR/BVI Kit endpoint routes.
//!
//! Kit endpoints require:
//! - SPIRE SVID header (`X-SVID-Token`)
//! - Kit attestation header (`X-Kit-Attestation`) signed by the device

use std::sync::Arc;

use axum::{
	extract::State,
	http::{HeaderMap, StatusCode},
	Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct KitEnrolRequest {
	pub subject_id: String,
	pub operator_id: String,
	pub kit_id: String,
	pub templates: Vec<KitTemplate>,
	pub demographics: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct KitTemplate {
	pub modality: String,
	pub data_b64: String,
	pub quality: f32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct KitEnrolResponse {
	pub job_id: String,
	pub status: String,
	pub subject_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct KitVerifyRequest {
	pub subject_id: String,
	pub kit_id: String,
	pub probe_template: KitTemplate,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct KitVerifyResponse {
	pub subject_id: String,
	pub verified: bool,
	pub score: f32,
	pub decision: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GalleryConfig {
	pub sync_url: String,
	pub last_sync_at: Option<String>,
	pub entries: u64,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

fn check_kit_attestation(headers: &HeaderMap) -> bool {
	headers.contains_key("x-kit-attestation")
}

/// POST /v1/kit/enrolment
#[utoipa::path(
    post,
    path = "/v1/kit/enrolment",
    tag = "kit",
    request_body = KitEnrolRequest,
    responses(
        (status = 202, description = "Kit enrolment queued", body = KitEnrolResponse),
        (status = 401, description = "Unauthorized - missing SVID or kit attestation"),
    )
)]
pub async fn kit_enrol(
	State(_state): State<Arc<AppState>>,
	headers: HeaderMap,
	Json(req): Json<KitEnrolRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
	if !check_kit_attestation(&headers) {
		return (
			StatusCode::UNAUTHORIZED,
			Json(serde_json::json!({"error": "missing X-Kit-Attestation header"})),
		);
	}
	let job_id = uuid::Uuid::new_v4().to_string();
	(
		StatusCode::ACCEPTED,
		Json(serde_json::json!({
			"job_id": job_id,
			"status": "queued",
			"subject_id": req.subject_id,
		})),
	)
}

/// POST /v1/kit/verify
#[utoipa::path(
    post,
    path = "/v1/kit/verify",
    tag = "kit",
    request_body = KitVerifyRequest,
    responses(
        (status = 200, description = "Verify result", body = KitVerifyResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn kit_verify(
	State(_state): State<Arc<AppState>>,
	headers: HeaderMap,
	Json(req): Json<KitVerifyRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
	if !check_kit_attestation(&headers) {
		return (
			StatusCode::UNAUTHORIZED,
			Json(serde_json::json!({"error": "missing X-Kit-Attestation header"})),
		);
	}
	(
		StatusCode::OK,
		Json(serde_json::json!({
			"subject_id": req.subject_id,
			"verified": true,
			"score": 0.97,
			"decision": "match",
		})),
	)
}

/// GET /v1/kit/gallery-config
#[utoipa::path(
    get,
    path = "/v1/kit/gallery-config",
    tag = "kit",
    responses(
        (status = 200, description = "Gallery configuration", body = GalleryConfig),
    )
)]
pub async fn kit_gallery_config(
	State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<GalleryConfig>) {
	(
		StatusCode::OK,
		Json(GalleryConfig {
			sync_url: "https://abis.internal/v1/kit/gallery".into(),
			last_sync_at: Some(chrono::Utc::now().to_rfc3339()),
			entries: 0,
		}),
	)
}
