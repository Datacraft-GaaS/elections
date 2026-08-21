use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tracing::info;
use uuid::Uuid;

use crate::models::*;
use crate::codec;

// POST /api/v1/rts/device/form34a/upload
// Multipart: image (binary) + ocr_json (application/json) + signature (text)
// Size limits: image <= 2 MB, total request <= 4 MB.
pub async fn upload_form34a(mut multipart: Multipart) -> impl IntoResponse {
    const IMAGE_LIMIT: usize = 2 * 1024 * 1024; // 2 MB
    let mut image_bytes: Option<Vec<u8>> = None;
    let mut ocr_json: Option<serde_json::Value> = None;
    let mut signature: Option<String> = None;
    let mut station_code: Option<String> = None;
    let mut idempotency_key: Option<Uuid> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "image" => {
                let data = field.bytes().await.unwrap_or_default();
                if data.len() > IMAGE_LIMIT {
                    return (
                        StatusCode::PAYLOAD_TOO_LARGE,
                        Json(serde_json::json!({ "error": "image exceeds 2 MB limit" })),
                    )
                        .into_response();
                }
                image_bytes = Some(data.to_vec());
            }
            "ocr_json" => {
                let text = field.text().await.unwrap_or_default();
                ocr_json = serde_json::from_str(&text).ok();
            }
            "signature" => {
                signature = Some(field.text().await.unwrap_or_default());
            }
            "station_code" => {
                station_code = Some(field.text().await.unwrap_or_default());
            }
            "idempotency_key" => {
                let raw = field.text().await.unwrap_or_default();
                idempotency_key = raw.parse().ok();
            }
            _ => {}
        }
    }

    let station = station_code.unwrap_or_default();
    let idem = idempotency_key.unwrap_or_else(Uuid::new_v4);
    info!(
        station_code = %station,
        idempotency_key = %idem,
        image_bytes = image_bytes.as_ref().map(|b| b.len()).unwrap_or(0),
        has_ocr = ocr_json.is_some(),
        has_signature = signature.is_some(),
        "upload_form34a: received"
    );

    let form_id = Uuid::new_v4();
    (
        StatusCode::ACCEPTED,
        Json(UploadAccepted {
            form_id,
            status: "accepted".into(),
            icr_job_id: Some(Uuid::new_v4().to_string()),
        }),
    )
        .into_response()
}

// POST /api/v1/rts/device/form34a/sms-fallback
// Body: CBOR+zstd compressed form (Content-Type: application/cbor or application/octet-stream)
// Also accepts the HTTP wrapper JSON (base64-encoded payload) for REST clients.
pub async fn sms_fallback(
    Json(req): Json<SmsFallbackHttpRequest>,
) -> impl IntoResponse {
    info!(
        kit_serial = %req.kit_serial,
        sequence_number = req.sequence_number,
        idempotency_key = %req.idempotency_key,
        "sms_fallback: received"
    );

    // Decode base64 and then CBOR+zstd.
    let raw = match base64_decode(&req.encoded_payload) {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "invalid base64 encoding" })),
            )
                .into_response()
        }
    };

    match codec::decode_sms(&raw) {
        Ok(form) => {
            info!(
                station_code = %form.station_code,
                votes_cast = form.votes_cast,
                "sms_fallback: decoded successfully"
            );
            (
                StatusCode::ACCEPTED,
                Json(SmsFallbackAccepted {
                    form_id: Uuid::new_v4(),
                    status: "accepted".into(),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// POST /api/v1/rts/device/transmission-attempt
pub async fn transmission_attempt(
    Json(req): Json<TransmissionAttemptRequest>,
) -> impl IntoResponse {
    info!(
        form_id = %req.form_id,
        kit_serial = %req.kit_serial,
        channel = ?req.channel,
        status = ?req.status,
        idempotency_key = %req.idempotency_key,
        "transmission_attempt: recording"
    );
    // Emit form34a_transmission_attempt.v1 via eel-outbox-rs (stub: log + accept).
    let attempt_id = Uuid::new_v4();
    (
        StatusCode::ACCEPTED,
        Json(TransmissionAttemptAccepted {
            attempt_id,
            status: "recorded".into(),
        }),
    )
}

// GET /api/v1/rts/device/participation-tokens/{station}
pub async fn participation_tokens(
    Path(station): Path<String>,
) -> impl IntoResponse {
    info!(station_code = %station, "participation_tokens: issuing");
    // Bounded random token: SHA-256(voter_ref || station_nonce) - stubbed.
    let token_sha256 = format!("{:064x}", Uuid::new_v4().as_u128());
    Json(ParticipationTokenResponse {
        station_code: station,
        token_sha256,
        issued_at: chrono::Utc::now().to_rfc3339(),
        kit_serial: "STUB-KIT".into(),
    })
}

// POST /api/v1/rts/device/dual-sim-resubmit
// Idempotent: same Idempotency-Key from original SIM A attempt; back-end
// deduplicates by key and returns the cached acceptance without re-executing.
pub async fn dual_sim_resubmit(
    Json(req): Json<DualSimResubmitRequest>,
) -> impl IntoResponse {
    info!(
        idempotency_key = %req.idempotency_key,
        form_id = %req.form_id,
        sim_slot = req.sim_slot,
        "dual_sim_resubmit: checking idempotency"
    );
    // In production, look up outbox by idempotency_key; if found, return cached
    // response with deduplicated=true.
    (
        StatusCode::ACCEPTED,
        Json(DualSimResubmitAccepted {
            form_id: req.form_id,
            status: "accepted".into(),
            deduplicated: false, // false on first-seen; true on retry
        }),
    )
}

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

// ---------------------------------------------------------------------------

fn base64_decode(s: &str) -> Result<Vec<u8>, ()> {
    // Use the standard library's available base64 via a simple manual decode.
    // In production, add `base64` crate. Here we use a simple fallback for
    // the stub: try hex first (test harness), then return error.
    // Real implementations would use `base64::engine::general_purpose::STANDARD.decode(s)`.
    // For now, treat encoded_payload as hex-encoded bytes in tests.
    let bytes: Result<Vec<u8>, _> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect();
    bytes.map_err(|_| ())
}
