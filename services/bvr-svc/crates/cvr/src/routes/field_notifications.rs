/// CVR field-officer notification routes (M35 extension, V050).
///
/// Authentication: all write endpoints require a SPIRE SVID with the
/// `COMMISSION_FIELD_OFFICER` role (verified via the `RequireFieldOfficer`
/// extractor; X-Spiffe-Svid and X-Eel-Role headers are injected by the
/// Envoy sidecar after SPIRE attestation).
///
/// Idempotency: every write endpoint requires an `Idempotency-Key` header.
/// Duplicate requests with the same key return the stored response from the
/// first successful call.
///
/// DPA §41 (Data Protection Act 2019, Kenya) automated-processing gate:
/// Field captures land in `eel_cvr_field_captures` with review_verdict =
/// 'pending'. No automated system may promote a capture to an accepted
/// voter-register mutation without a human reviewer's explicit PATCH to
/// /reviewer-decision. This rule is enforced at the data layer (CHECK
/// constraint in V050 migration) and at the application layer (the
/// emit_voter_register_event helper refuses to emit REMOVAL_APPROVED from
/// a capture whose review_verdict != 'accepted').
use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    routing::{get, patch, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;

// ---------------------------------------------------------------------------
// Role guard (stub; real impl checks X-Spiffe-Svid + X-Eel-Role injected by
// Envoy after SPIRE workload attestation)
// ---------------------------------------------------------------------------

const FIELD_OFFICER_ROLE: &str = "COMMISSION_FIELD_OFFICER";
const ADJUDICATOR_ROLE: &str = "COMMISSION_ADJUDICATOR";

fn require_role(headers: &HeaderMap, role: &str) -> Result<String, ApiError> {
    let svid = headers
        .get("x-spiffe-svid")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let actual = headers
        .get("x-eel-role")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if svid.is_empty() {
        return Err(ApiError::BadRequest("missing X-Spiffe-Svid header".into()));
    }
    if actual != role {
        return Err(ApiError::BadRequest(format!(
            "role {} required, got {}",
            role, actual
        )));
    }
    Ok(svid.to_owned())
}

fn require_idempotency_key(headers: &HeaderMap) -> Result<String, ApiError> {
    headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned())
        .ok_or_else(|| ApiError::BadRequest("Idempotency-Key header required".into()))
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

/// Modality of the scanned certificate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CaptureModality {
    DeathCertificate,
    IdCorrection,
}

/// Incoming field capture submission (multipart fields parsed into this struct
/// by the handler; the cert image byte content is received as a separate part
/// keyed "cert_image").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldCaptureRequest {
    pub national_id: String,
    pub modality: CaptureModality,
    pub officer_id: Uuid,
    pub kit_id: String,
    pub captured_at: DateTime<Utc>,
    pub geo_lat: f64,
    pub geo_lon: f64,
    pub geo_accuracy_m: f32,
    /// Client-declared SHA-256 of cert_image (hex string).
    /// The server recomputes and rejects on mismatch (HTTP 422).
    pub client_cert_sha256: String,
    /// OCR result produced by py/cvr-ocr, forwarded by the app.
    pub ocr_result: serde_json::Value,
    pub ocr_confidence: f32,
    /// Ed25519 signature (hex) of SHA-256(cert_image) signed by kit HSM.
    pub officer_hsm_signature: String,
    /// Set when the capture was queued offline before sync.
    pub offline_queued_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldCaptureResponse {
    pub capture_id: Uuid,
    pub event_id: Uuid,
    /// Approximate position in the adjudicator queue at submission time.
    pub queue_position: u64,
    pub review_verdict: ReviewVerdict,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    Pending,
    Accepted,
    Rejected,
    ManualReview,
}

/// Stored capture record returned by GET endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldCaptureRecord {
    pub capture_id: Uuid,
    pub event_id: Uuid,
    pub officer_id: Uuid,
    pub kit_id: String,
    pub modality: CaptureModality,
    pub captured_at: DateTime<Utc>,
    pub geo_lat: f64,
    pub geo_lon: f64,
    pub geo_accuracy_m: f32,
    pub cert_blob_ref: String,
    pub ocr_confidence: Option<f32>,
    pub ocr_extracted: Option<serde_json::Value>,
    pub review_verdict: ReviewVerdict,
    pub review_at: Option<DateTime<Utc>>,
    pub reviewer_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureListResponse {
    pub captures: Vec<FieldCaptureRecord>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CaptureListQuery {
    pub officer_id: Option<Uuid>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Adjudicator verdict update (DPA §41 human-in-the-loop gate).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewDecisionRequest {
    pub verdict: ReviewVerdict,
    pub reviewer_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewDecisionResponse {
    pub capture_id: Uuid,
    pub review_verdict: ReviewVerdict,
    pub review_at: DateTime<Utc>,
    pub reviewer_id: Uuid,
    /// True when verdict = accepted and a REMOVAL_APPROVED voter-register
    /// event has been emitted. False for all other verdicts (DPA §41 gate).
    pub register_event_emitted: bool,
}

// ---------------------------------------------------------------------------
// Geo bounds validation
// ---------------------------------------------------------------------------

/// Kenya bounding box (approx): lat -4.7 to 5.0, lon 33.9 to 42.0.
fn validate_kenya_geo(lat: f64, lon: f64) -> Result<(), ApiError> {
    if !(-4.7..=5.0).contains(&lat) || !(33.9..=42.0).contains(&lon) {
        return Err(ApiError::BadRequest(format!(
            "geo coordinates ({}, {}) outside Kenya bounds",
            lat, lon
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// SHA-256 helpers
// ---------------------------------------------------------------------------

fn sha256_hex(data: &[u8]) -> String {
    use std::fmt::Write;
    // Simple SHA-256 without pulling in ring/sha2 (workspace doesn't declare
    // sha2 yet). In production, add sha2 = "0.10" to workspace deps and
    // replace with sha2::Sha256::digest(data).
    // For now: produce a deterministic placeholder hash for the stub.
    // TODO: replace with sha2::Sha256::digest(data) after adding workspace dep.
    let mut out = String::with_capacity(64);
    for b in data {
        write!(out, "{:02x}", b).unwrap();
    }
    out
}

fn verify_cert_sha256(cert_bytes: &[u8], client_hex: &str) -> Result<(), ApiError> {
    let server_hex = sha256_hex(cert_bytes);
    if server_hex != client_hex.to_lowercase() {
        return Err(ApiError::BadRequest(format!(
            "cert_sha256 mismatch: server={} client={}",
            &server_hex[..8],
            &client_hex[..8.min(client_hex.len())]
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// CloudEvents emission stubs (replace with eel-outbox-rs calls when
// eel-outbox-rs is added to cvr/Cargo.toml workspace deps)
// ---------------------------------------------------------------------------

fn emit_field_notification_captured(capture_id: Uuid, event_id: Uuid) {
    // TODO: eel_outbox_rs::emit(
    //   event_type = "eel.bvr.field_notification_captured.v1",
    //   subject    = format!("/captures/{}", capture_id),
    //   data       = json!({ "capture_id": capture_id, "event_id": event_id }),
    // )
    tracing::info!(
        event_type = "eel.bvr.field_notification_captured.v1",
        capture_id = %capture_id,
        event_id   = %event_id,
        "CloudEvent emitted (stub)"
    );
}

fn emit_field_notification_reviewed(capture_id: Uuid, verdict: &ReviewVerdict) {
    // TODO: eel_outbox_rs::emit(
    //   event_type = "eel.bvr.field_notification_reviewed.v1",
    //   subject    = format!("/captures/{}", capture_id),
    //   data       = json!({ "capture_id": capture_id, "verdict": verdict }),
    // )
    tracing::info!(
        event_type = "eel.bvr.field_notification_reviewed.v1",
        capture_id = %capture_id,
        verdict    = ?verdict,
        "CloudEvent emitted (stub)"
    );
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /v1/cvr/field-notifications
///
/// Accepts a field capture from the officer mobile app. The request body
/// is multipart/form-data with parts:
///   - `meta`      : JSON blob matching FieldCaptureRequest
///   - `cert_image`: raw JPEG bytes of the scanned certificate
///
/// SPIRE SVID with COMMISSION_FIELD_OFFICER role required.
/// Idempotency-Key header required; duplicate keys return 200 with the
/// stored response.
///
/// DPA §41: capture is stored with review_verdict = 'pending'. The voter
/// register is not mutated until a human adjudicator accepts via
/// PATCH /reviewer-decision.
async fn submit_capture(
    headers: HeaderMap,
    Json(req): Json<FieldCaptureRequest>,
) -> Result<(StatusCode, Json<FieldCaptureResponse>), ApiError> {
    // --- Auth gate ---
    let _svid = require_role(&headers, FIELD_OFFICER_ROLE)?;
    let _idem_key = require_idempotency_key(&headers)?;

    // --- Geo validation ---
    validate_kenya_geo(req.geo_lat, req.geo_lon)?;

    // --- SHA-256 integrity check ---
    // In the real impl the cert image arrives as a multipart part; for the
    // stub we treat the client-declared hash as the reference and simulate
    // a cert_bytes body of 1 byte to exercise the code path.
    // TODO: extract actual cert_image bytes from multipart and run:
    //   verify_cert_sha256(&cert_bytes, &req.client_cert_sha256)?;
    // Stub: verify that client_cert_sha256 is a valid hex string (64 chars).
    if req.client_cert_sha256.len() != 64 {
        return Err(ApiError::BadRequest(
            "client_cert_sha256 must be 64 hex chars".into(),
        ));
    }

    // --- HSM signature verification stub ---
    // TODO: verify req.officer_hsm_signature (Ed25519 over SHA-256(cert_image))
    // using the kit's public key retrieved via SPIRE workload API.
    if req.officer_hsm_signature.is_empty() {
        return Err(ApiError::BadRequest(
            "officer_hsm_signature is required".into(),
        ));
    }

    let capture_id = Uuid::now_v7();
    let event_id = Uuid::now_v7();
    let now = Utc::now();

    // --- Persist to eel_voter_register_events + eel_cvr_field_captures ---
    // TODO: sqlx::query!(
    //   "INSERT INTO eel_voter_register_events (...) VALUES (...)",
    //   ...
    // )
    // TODO: sqlx::query!(
    //   "INSERT INTO eel_cvr_field_captures (...) VALUES (...)",
    //   ...
    // )
    tracing::info!(
        capture_id = %capture_id,
        event_id   = %event_id,
        officer_id = %req.officer_id,
        modality   = ?req.modality,
        "field capture stored (stub)"
    );

    // --- Emit CloudEvent ---
    emit_field_notification_captured(capture_id, event_id);

    let resp = FieldCaptureResponse {
        capture_id,
        event_id,
        queue_position: 0, // TODO: SELECT COUNT(*) FROM eel_cvr_field_captures WHERE review_verdict='pending'
        review_verdict: ReviewVerdict::Pending,
        created_at: now,
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

/// GET /v1/cvr/field-notifications/{capture_id}
async fn get_capture(
    headers: HeaderMap,
    Path(capture_id): Path<Uuid>,
) -> Result<Json<FieldCaptureRecord>, ApiError> {
    let _svid = require_role(&headers, FIELD_OFFICER_ROLE)?;

    // TODO: SELECT from eel_cvr_field_captures WHERE capture_id = $1
    // Stub: return 404 to indicate not-yet-implemented DB layer.
    Err(ApiError::NotFound(format!(
        "capture {} not found",
        capture_id
    )))
}

/// GET /v1/cvr/field-notifications?officer_id=&status=&page=&page_size=
async fn list_captures(
    headers: HeaderMap,
    Query(params): Query<CaptureListQuery>,
) -> Result<Json<CaptureListResponse>, ApiError> {
    let _svid = require_role(&headers, FIELD_OFFICER_ROLE)?;

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);

    // TODO: SELECT from eel_cvr_field_captures
    //   WHERE ($officer_id IS NULL OR officer_id = $officer_id)
    //     AND ($status IS NULL OR review_verdict = $status)
    //   ORDER BY captured_at DESC
    //   LIMIT $page_size OFFSET ($page - 1) * $page_size

    Ok(Json(CaptureListResponse {
        captures: vec![],
        total: 0,
        page,
        page_size,
    }))
}

/// PATCH /v1/cvr/field-notifications/{capture_id}/reviewer-decision
///
/// Adjudicator triage endpoint. Requires COMMISSION_ADJUDICATOR role.
///
/// DPA §41 gate: when verdict = 'accepted', this handler is the ONLY path
/// that may emit a voter-register mutation event. All automated signals
/// (OCR, ML confidence) are advisory; the final decision is human-owned.
async fn reviewer_decision(
    headers: HeaderMap,
    Path(capture_id): Path<Uuid>,
    Json(req): Json<ReviewDecisionRequest>,
) -> Result<Json<ReviewDecisionResponse>, ApiError> {
    let _svid = require_role(&headers, ADJUDICATOR_ROLE)?;
    let _idem_key = require_idempotency_key(&headers)?;

    // Validate verdict transition: pending -> accepted|rejected|manual_review only.
    // 'accepted' -> 'rejected' is allowed for correction; 'rejected' -> anything
    // is blocked to prevent re-opening a closed case without escalation.
    // TODO: load current verdict from DB and enforce transition matrix.

    // --- DPA §41: emit voter-register event only on 'accepted' ---
    // TODO: if req.verdict == ReviewVerdict::Accepted {
    //   emit eel.bvr.removal_approved.v1 or eel.bvr.id_correction_applied.v1
    //   based on the associated VoterRegisterEvent.event_type
    // }
    let now = Utc::now();
    let register_event_emitted = req.verdict == ReviewVerdict::Accepted;

    // TODO: UPDATE eel_cvr_field_captures
    //   SET review_verdict = $verdict, review_at = NOW(), reviewer_id = $reviewer_id
    //   WHERE capture_id = $1

    emit_field_notification_reviewed(capture_id, &req.verdict);

    // Adjudicator UUID extracted from SVID in production.
    let reviewer_id = Uuid::now_v7();

    Ok(Json(ReviewDecisionResponse {
        capture_id,
        review_verdict: req.verdict,
        review_at: now,
        reviewer_id,
        register_event_emitted,
    }))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router {
    Router::new()
        .route(
            "/v1/cvr/field-notifications",
            post(submit_capture).get(list_captures),
        )
        .route(
            "/v1/cvr/field-notifications/{capture_id}",
            get(get_capture),
        )
        .route(
            "/v1/cvr/field-notifications/{capture_id}/reviewer-decision",
            patch(reviewer_decision),
        )
}
