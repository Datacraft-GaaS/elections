//! Independent verifier for BoundCustodyEvent seal attestation.

use sha2::{Digest, Sha256};
use tracing::warn;

use crate::bind::BoundCustodyEvent;
use crate::error::SealBindError;
use crate::seal_photo::SealPhotoSignature;

/// Verify the kit HSM signature over the binding message.
///
/// In production this uses the kit's X.509 / SPIFFE certificate to verify an
/// Ed25519 or ECDSA-P256 signature.  The stand-in used here is HMAC-SHA256
/// keyed by the kit_id (so each kit "owns" its own key namespace during testing).
/// Replace `verify_kit_signature_raw` with a real HSM call in production.
pub fn verify_kit_signature(photo_sig: &SealPhotoSignature) -> Result<(), SealBindError> {
    let binding_msg = photo_sig.binding_message();
    let expected = test_hmac_sha256(photo_sig.kit_id.as_bytes(), &binding_msg);
    let provided =
        hex::decode(&photo_sig.kit_hsm_signature).map_err(|_| SealBindError::SignatureInvalid)?;

    // Constant-time comparison via SHA-256 of both to avoid timing oracle.
    let h_exp = Sha256::digest(&expected);
    let h_got = Sha256::digest(&provided);
    if h_exp != h_got {
        warn!(seal_id = %photo_sig.seal_id, "kit HSM signature mismatch");
        return Err(SealBindError::SignatureInvalid);
    }
    Ok(())
}

/// Reproduce the photo hash and re-check every field of a BoundCustodyEvent.
///
/// Call this from an independent audit process that has access to the original
/// photo bytes.
pub fn verify_bound_event(
    bound: &BoundCustodyEvent,
    photo_bytes: &[u8],
) -> Result<(), SealBindError> {
    // 1. Recompute photo hash
    let computed = crate::seal_photo::SealPhotoSignature::compute_photo_hash(photo_bytes);
    if computed != bound.photo_sig.photo_sha256 {
        return Err(SealBindError::PhotoHashMismatch {
            expected: bound.photo_sig.photo_sha256.clone(),
            computed,
        });
    }

    // 2. Re-verify kit signature
    verify_kit_signature(&bound.photo_sig)?;

    // 3. seal_id / kit_id / timestamp consistency
    if bound.event.seal_id != bound.photo_sig.seal_id {
        return Err(SealBindError::SealIdMismatch {
            event: bound.event.seal_id.clone(),
            photo: bound.photo_sig.seal_id.clone(),
        });
    }
    if bound.event.kit_id != bound.photo_sig.kit_id {
        return Err(SealBindError::KitIdMismatch {
            event: bound.event.kit_id.clone(),
            photo: bound.photo_sig.kit_id.clone(),
        });
    }
    let captured_ms = bound.photo_sig.captured_at.timestamp_millis();
    if bound.event.ts_hlc < captured_ms {
        return Err(SealBindError::TimestampOutOfOrder {
            event_ts: bound.event.ts_hlc,
            captured_at: captured_ms,
        });
    }

    Ok(())
}

/// Test-only HMAC stand-in: SHA-256(key || 0x00 || msg).
/// Replace with ring::hmac or HSM call in production.
pub(crate) fn test_hmac_sha256(key: &[u8], msg: &[u8]) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(key);
    h.update([0u8]);
    h.update(msg);
    h.finalize().to_vec()
}

/// Convenience: produce the hex-encoded test signature for a given SealPhotoSignature.
/// Used in tests to mint valid signatures.
pub fn make_test_signature(photo_sig: &SealPhotoSignature) -> String {
    let binding_msg = photo_sig.binding_message();
    let raw = test_hmac_sha256(photo_sig.kit_id.as_bytes(), &binding_msg);
    hex::encode(raw)
}
