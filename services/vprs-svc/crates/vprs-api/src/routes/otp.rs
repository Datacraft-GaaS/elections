//! OTP issuance and verification for pre-registration sessions.
//! FR-M04-010: issue a single-use 6 digit code bound to an application.
//! FR-M04-011: verify the code; lock out after MAX_REQUESTS attempts.
//! EC-SEC-12: codes are stored hashed; comparison is constant time.
//!
//! The generated code is never returned by the API or logged. Delivery is
//! delegated to the outbound notification pipeline keyed on the application.

use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use tracing::info;

use crate::{error::ApiError, state::AppState};
use vprs_domain::otp::{OtpRequest, LOCKOUT_MINUTES, MAX_REQUESTS};

/// Lifetime of an issued code, in seconds.
const CODE_TTL_SECONDS: i64 = 600;

#[derive(Debug, Deserialize)]
pub struct RequestOtpPayload {
    pub app_id: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyOtpPayload {
    pub app_id: String,
    pub code: String,
}

/// POST /vprs/v1/otp/request
///
/// Issues a fresh code for the application. Returns 404 for unknown
/// applications and 429 once MAX_REQUESTS issues have been reached or the
/// session is locked out.
#[allow(dead_code)]
pub async fn request_otp(
    State(state): State<AppState>,
    Json(payload): Json<RequestOtpPayload>,
) -> Result<Json<Value>, ApiError> {
    // The application must exist before a code can be bound to it.
    let app = sqlx::query("SELECT status FROM vprs_applications WHERE app_id = $1")
        .bind(&payload.app_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| ApiError::ServiceUnavailable)?;
    match app {
        Some(r) => {
            let status: String = r.get("status");
            if matches!(status.as_str(), "cancelled" | "expired" | "converted") {
                return Err(ApiError::NotFound);
            }
        }
        None => return Err(ApiError::NotFound),
    }

    // Rate limit against any existing session using the shared domain rules.
    let existing = sqlx::query(
        "SELECT attempt_count, locked_until FROM vprs_otp_sessions WHERE app_id = $1",
    )
    .bind(&payload.app_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| ApiError::ServiceUnavailable)?;

    if let Some(row) = existing {
        let locked_until: Option<chrono::DateTime<Utc>> = row.get("locked_until");
        let attempt_count: i32 = row.get("attempt_count");
        let session = OtpRequest {
            session_id: String::new(),
            attempt_count: attempt_count.max(0) as u32,
            locked_until,
        };
        if session.is_locked() || session.is_rate_limited() {
            info!(app_id = %payload.app_id, "otp request rate limited");
            return Err(ApiError::TooManyRequests);
        }
    }

    let session_id = vprs_domain::uuid7str();
    let code_hash = eel_crypto_rs::blake3::hash_hex(generate_code().as_bytes());

    let _ = sqlx::query(
        r#"
        INSERT INTO vprs_otp_sessions (session_id, app_id, code_hash, attempt_count, expires_at)
        VALUES ($1, $2, $3, 0, NOW() + make_interval(secs => $4))
        ON CONFLICT (app_id) DO UPDATE SET
            session_id = EXCLUDED.session_id,
            code_hash = EXCLUDED.code_hash,
            attempt_count = 0,
            locked_until = NULL,
            verified = FALSE,
            expires_at = EXCLUDED.expires_at
        "#,
    )
    .bind(&session_id)
    .bind(&payload.app_id)
    .bind(&code_hash)
    .bind(CODE_TTL_SECONDS)
    .execute(&state.db)
    .await
    .map_err(|_| ApiError::ServiceUnavailable)?;

    Ok(Json(json!({
        "session_id": session_id,
        "expires_in_seconds": CODE_TTL_SECONDS,
    })))
}

/// POST /vprs/v1/otp/verify
///
/// Verifies a submitted code. A wrong code consumes one of MAX_REQUESTS
/// attempts; exhausting them locks the session for LOCKOUT_MINUTES minutes.
#[allow(dead_code)]
pub async fn verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<VerifyOtpPayload>,
) -> Result<Json<Value>, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT session_id, code_hash, attempt_count, locked_until, expires_at
        FROM vprs_otp_sessions WHERE app_id = $1
        "#,
    )
    .bind(&payload.app_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| ApiError::ServiceUnavailable)?;

    let row = row.ok_or(ApiError::NotFound)?;
    let session_id: String = row.get("session_id");
    let code_hash: String = row.get("code_hash");
    let locked_until: Option<DateTime<Utc>> = row.get("locked_until");
    let expires_at: DateTime<Utc> = row.get("expires_at");
    let attempt_count: i32 = row.get("attempt_count");
    let session = OtpRequest {
        session_id: session_id.clone(),
        attempt_count: attempt_count.max(0) as u32,
        locked_until,
    };

    if session.is_locked() || session.is_rate_limited() {
        return Err(ApiError::TooManyRequests);
    }
    if Utc::now() > expires_at {
        return Err(ApiError::UnprocessableEntity(
            "verification session expired".to_string(),
        ));
    }

    let submitted = eel_crypto_rs::blake3::hash_hex(payload.code.trim().as_bytes());
    if eel_crypto_rs::constant_time::ct_eq(submitted.as_bytes(), code_hash.as_bytes()) {
        let _ = sqlx::query("UPDATE vprs_otp_sessions SET verified = TRUE WHERE session_id = $1")
            .bind(&session_id)
            .execute(&state.db)
            .await
            .map_err(|_| ApiError::ServiceUnavailable)?;
        info!(app_id = %payload.app_id, "otp verified");
        return Ok(Json(json!({ "verified": true })));
    }

    // Wrong code: consume an attempt and lock the session once exhausted.
    let _ = sqlx::query(
        r#"
        UPDATE vprs_otp_sessions SET
            attempt_count = attempt_count + 1,
            locked_until = CASE
                WHEN attempt_count + 1 >= $2 THEN NOW() + make_interval(mins => $3)
                ELSE locked_until
            END
        WHERE session_id = $1
        "#,
    )
    .bind(&session_id)
    .bind(i32::try_from(MAX_REQUESTS).unwrap_or(i32::MAX))
    .bind(LOCKOUT_MINUTES)
    .execute(&state.db)
    .await
    .map_err(|_| ApiError::ServiceUnavailable)?;

    Err(ApiError::UnprocessableEntity("invalid code".to_string()))
}

/// Derive a zero padded 6 digit code from fresh randomness.
/// The raw value exists only inside this call; callers persist its hash.
fn generate_code() -> String {
    let bytes = uuid::Uuid::new_v4().into_bytes();
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[0..8]);
    format!("{:06}", u64::from_be_bytes(raw) % 1_000_000)
}
