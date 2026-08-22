//! SealPhotoSignature: the kit's signed record of a captured seal photo.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Signed record produced by the kit after photographing a seal at a custody handover.
///
/// The kit computes `photo_sha256 = SHA-256(photo_bytes)` and then signs
/// `SHA-256(seal_id_bytes || kit_id_bytes || photo_sha256)` with its HSM private key.
/// The resulting `kit_hsm_signature` is stored as lowercase hex.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealPhotoSignature {
    /// Barcode or RFID serial of the physical seal.
    pub seal_id: String,
    /// SHA-256 of the raw photo bytes (hex-encoded, lowercase).
    pub photo_sha256: String,
    /// ID of the kit that captured the photo.
    pub kit_id: String,
    /// UTC timestamp when the photo was captured.
    pub captured_at: DateTime<Utc>,
    /// Kit HSM ECDSA/EdDSA signature over the binding message (hex-encoded).
    /// In production this is an Ed25519 or ECDSA-P256 signature from the kit's
    /// factory-provisioned private key.  In tests, HMAC-SHA256 with a shared
    /// secret is used as a stand-in so no HSM hardware is required.
    pub kit_hsm_signature: String,
    /// WGS-84 latitude at capture time.
    pub geo_lat: f64,
    /// WGS-84 longitude at capture time.
    pub geo_lon: f64,
}

impl SealPhotoSignature {
    /// Compute the binding message that the kit signs:
    /// `SHA-256(seal_id_utf8 || 0x00 || kit_id_utf8 || 0x00 || photo_sha256_hex_utf8)`
    pub fn binding_message(&self) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(self.seal_id.as_bytes());
        h.update([0u8]);
        h.update(self.kit_id.as_bytes());
        h.update([0u8]);
        h.update(self.photo_sha256.as_bytes());
        h.finalize().into()
    }

    /// Compute the SHA-256 of raw photo bytes.
    pub fn compute_photo_hash(photo_bytes: &[u8]) -> String {
        let digest = Sha256::digest(photo_bytes);
        hex::encode(digest)
    }
}
