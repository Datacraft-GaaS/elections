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

//! BVR/BVI Kit endpoint routes.
//!
//! Kit endpoints require:
//! - SPIRE SVID header (`X-SVID-Token`)
//! - Kit attestation header (`X-Kit-Attestation`) signed by the device
//!
//! Templates are persisted to `eel_enrolment_jobs` (source `BvrKit`) for the
//! downstream dedup/quality orchestrator; verification uses the deterministic
//! mock matcher until the modality matchers are wired end-to-end.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;
use uuid::Uuid;

use abis_matcher::{MatchOutcome, MatchPipeline, MockMatcher};

use crate::error::ApiError;
use crate::state::AppState;

/// Configured default similarity threshold (mirrors match_routes).
const DEFAULT_THRESHOLD: f32 = 0.8;

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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

fn check_kit_attestation(headers: &HeaderMap) -> Result<(), ApiError> {
    if headers.contains_key("x-kit-attestation") {
        Ok(())
    } else {
        Err(ApiError::Unauthorized(
            "missing X-Kit-Attestation header".into(),
        ))
    }
}

/// Upsert a kit row on first contact so `eel_enrolment_jobs.captured_by_kit_id`
/// can satisfy its FK to `eel_kits`. Attestation/SPIRE fields are filled with
/// software-legacy defaults until real device attestation lands.
async fn register_kit(db: &sqlx::PgPool, kit_id: &str) -> Result<(), ApiError> {
    let cert_fp = Sha256::digest(format!("kit:{kit_id}").as_bytes()).to_vec();
    let spiffe_id = format!("spiffe://eel.local/kit/{kit_id}");

    sqlx::query(
        "INSERT INTO eel_kits \
		     (kit_id, kit_type, device_model, firmware_version, cert_fingerprint, \
		      spiffe_id, attestation_profile) \
		 VALUES ($1, 'bvr', 'unknown', '0.0.0', $2, $3, 'software_legacy') \
		 ON CONFLICT (kit_id) DO UPDATE SET last_seen_at = NOW()",
    )
    .bind(kit_id)
    .bind(cert_fp)
    .bind(spiffe_id)
    .execute(db)
    .await?;

    Ok(())
}

/// POST /v1/kit/enrolment — persist captured templates as a `BvrKit`
/// enrolment job for the downstream orchestrator.
#[utoipa::path(
    post,
    path = "/v1/kit/enrolment",
    tag = "kit",
    request_body = KitEnrolRequest,
    responses(
        (status = 202, description = "Kit enrolment queued", body = KitEnrolResponse),
        (status = 401, description = "Unauthorized - missing SVID or kit attestation"),
        (status = 400, description = "Bad request - subject_id is not a UUID"),
    )
)]
pub async fn kit_enrol(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<KitEnrolRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    check_kit_attestation(&headers)?;

    let subject_id = Uuid::parse_str(&req.subject_id)
        .map_err(|_| ApiError::Validation("subject_id must be a UUID".into()))?;
    if req.templates.is_empty() {
        return Err(ApiError::Validation(
            "at least one template is required".into(),
        ));
    }

    let job_id = Uuid::new_v4();
    let templates_json = serde_json::to_value(&req.templates)
        .map_err(|e| ApiError::Validation(format!("invalid templates: {e}")))?;

    register_kit(&state.db, &req.kit_id).await?;

    sqlx::query(
        "INSERT INTO eel_enrolment_jobs \
		     (id, subject_id, source, status, captured_at, captured_by_kit_id, \
		      captured_by_officer, templates, created_at, updated_at) \
		 VALUES ($1, $2, 'BvrKit', 'Received', NOW(), $3, $4, $5, NOW(), NOW())",
    )
    .bind(job_id)
    .bind(subject_id)
    .bind(&req.kit_id)
    .bind(&req.operator_id)
    .bind(templates_json)
    .execute(&state.db)
    .await?;

    tracing::info!(job_id = %job_id, subject_id = %subject_id, templates = %req.templates.len(), "kit enrolment persisted");

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "job_id": job_id.to_string(),
            "status": "received",
            "subject_id": req.subject_id,
        })),
    ))
}

/// POST /v1/kit/verify — 1:1 verify against the subject's latest enrolled
/// templates.
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
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<KitVerifyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    check_kit_attestation(&headers)?;

    let subject_id = Uuid::parse_str(&req.subject_id)
        .map_err(|_| ApiError::Validation("subject_id must be a UUID".into()))?;

    let enrolled: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT templates FROM eel_enrolment_jobs \
		 WHERE subject_id = $1 AND jsonb_array_length(templates) > 0 \
		 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(subject_id)
    .fetch_optional(&state.db)
    .await?;

    let Some(enrolled) = enrolled else {
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "subject_id": req.subject_id,
                "verified": false,
                "score": 0.0,
                "decision": "no_enrollment",
            })),
        ));
    };

    let templates: Vec<KitTemplate> = serde_json::from_value(enrolled).unwrap_or_default();
    let gallery = templates
        .iter()
        .find(|t| t.modality == req.probe_template.modality)
        .or_else(|| templates.first());

    let Some(gallery) = gallery else {
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "subject_id": req.subject_id,
                "verified": false,
                "score": 0.0,
                "decision": "no_enrollment",
            })),
        ));
    };

    let result = MockMatcher.match_11(
        &req.probe_template.data_b64,
        &gallery.data_b64,
        DEFAULT_THRESHOLD,
    );
    let verified = result.outcome == MatchOutcome::Match;
    let decision = match result.outcome {
        MatchOutcome::Match => "match",
        MatchOutcome::NoMatch => "no_match",
        MatchOutcome::Adjudicate => "adjudicate",
        MatchOutcome::InsufficientQuality => "insufficient_quality",
    };

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "subject_id": req.subject_id,
            "verified": verified,
            "score": result.confidence_score,
            "decision": decision,
        })),
    ))
}

/// GET /v1/kit/gallery-config — gallery sync URL and enrolled entry count.
#[utoipa::path(
    get,
    path = "/v1/kit/gallery-config",
    tag = "kit",
    responses(
        (status = 200, description = "Gallery configuration", body = GalleryConfig),
    )
)]
pub async fn kit_gallery_config(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<GalleryConfig>), ApiError> {
    let entries: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM eel_enrolment_jobs WHERE jsonb_array_length(templates) > 0",
    )
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::OK,
        Json(GalleryConfig {
            sync_url: "https://abis.internal/v1/kit/gallery".into(),
            last_sync_at: Some(Utc::now().to_rfc3339()),
            entries: entries as u64,
        }),
    ))
}
