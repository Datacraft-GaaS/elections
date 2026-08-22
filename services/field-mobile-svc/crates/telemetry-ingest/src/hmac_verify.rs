//! HMAC-SHA256 verification over canonical signal payload.
//!
//! The per-device secret is referenced by `hmac_key_hsm_ref` on the device row.
//! In production this key is fetched from the HSM; in tests a direct secret is passed.

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::TelemetryError;

type HmacSha256 = Hmac<Sha256>;

/// Verify the HMAC-SHA256 signature over the canonical JSON of `payload`.
///
/// `secret` is the raw HMAC key bytes (fetched from HSM by the caller).
/// `signature` is the device-provided HMAC bytes to verify.
pub fn verify_hmac(
    secret: &[u8],
    payload: &serde_json::Value,
    signature: &[u8],
    device_id: &str,
) -> Result<(), TelemetryError> {
    let canonical =
        serde_json::to_vec(payload).map_err(|e| TelemetryError::Internal(e.to_string()))?;

    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| {
        TelemetryError::InvalidHmacKeyRef("key length invalid for HMAC-SHA256".to_owned())
    })?;
    mac.update(&canonical);
    mac.verify_slice(signature)
        .map_err(|_| TelemetryError::HmacFailure {
            device_id: device_id.to_owned(),
        })
}

/// Compute HMAC-SHA256 over canonical JSON of `payload`. Used by server counter-sign.
pub fn compute_hmac(secret: &[u8], payload: &serde_json::Value) -> Result<Vec<u8>, TelemetryError> {
    let canonical =
        serde_json::to_vec(payload).map_err(|e| TelemetryError::Internal(e.to_string()))?;

    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|_| TelemetryError::InvalidHmacKeyRef("key length invalid".to_owned()))?;
    mac.update(&canonical);
    Ok(mac.finalize().into_bytes().to_vec())
}
