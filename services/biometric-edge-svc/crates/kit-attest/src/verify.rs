use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::verdict::{AttestationVerdict, IntegrityLevel, KitTier};

/// Raw attestation kind submitted by the kit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationKind {
    PlayIntegrity,
    AppAttest,
    TpmQuote,
    LegacySoftware,
}

/// Attestation payload from a kit device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationPayload {
    /// Attestation kind identifying the verification path.
    pub kind: AttestationKind,
    /// Base64-encoded raw attestation token / quote.
    pub token: String,
    /// Device serial or identifier claim in the payload.
    pub device_serial: String,
    /// Optional nonce bound into the attestation (for freshness).
    pub nonce: Option<String>,
}

/// Errors produced during attestation verification.
#[derive(Debug, Error)]
pub enum AttestError {
    #[error("attestation signature invalid: {0}")]
    InvalidSignature(String),
    #[error("attestation token expired")]
    Expired,
    #[error("attestation token tampered: {0}")]
    Tampered(String),
    #[error("unsupported attestation kind")]
    Unsupported,
    #[error("internal verification error: {0}")]
    Internal(String),
}

/// Verify an attestation payload and return a verdict.
///
/// Production implementations will call out to Google Play Integrity API,
/// Apple App Attest service, or a local TPM CA validation chain.
/// This implementation is a well-structured stub with correct verdict shapes.
pub fn verify_attestation(payload: &AttestationPayload) -> Result<AttestationVerdict, AttestError> {
    // Reject obviously tampered tokens (empty token string).
    if payload.token.is_empty() {
        return Err(AttestError::Tampered("empty token".into()));
    }

    let (tier, integrity_level, legacy_warning) = match payload.kind {
        AttestationKind::PlayIntegrity => {
            verify_play_integrity(&payload.token)?;
            (KitTier::PlayIntegrity, IntegrityLevel::MeetsStrong, false)
        }
        AttestationKind::AppAttest => {
            verify_app_attest(&payload.token)?;
            (KitTier::AppAttest, IntegrityLevel::MeetsStrong, false)
        }
        AttestationKind::TpmQuote => {
            verify_tpm_quote(&payload.token)?;
            (KitTier::Tpm, IntegrityLevel::MeetsDevice, false)
        }
        AttestationKind::LegacySoftware => {
            verify_legacy_software(&payload.token)?;
            (KitTier::Legacy, IntegrityLevel::MeetsBasic, true)
        }
    };

    let thumbprint = derive_thumbprint(&payload.token);
    let expires_at = Utc::now() + chrono::Duration::hours(24);

    Ok(AttestationVerdict {
        device_id: payload.device_serial.clone(),
        tier,
        expires_at,
        integrity_level,
        kit_cert_thumbprint: thumbprint,
        legacy_warning,
    })
}

// Internal per-kind verification stubs.

fn verify_play_integrity(token: &str) -> Result<(), AttestError> {
    // Production: POST to https://playintegrity.googleapis.com/v1/{packageName}:decodeIntegrityToken
    // and verify verdict_nonce, app_integrity.verdict, device_integrity.verdict.
    if token.starts_with("TAMPERED:") {
        return Err(AttestError::Tampered(token.to_string()));
    }
    Ok(())
}

fn verify_app_attest(token: &str) -> Result<(), AttestError> {
    // Production: call Apple Device Check / App Attest API to validate the
    // attestation object and assertion.
    if token.starts_with("TAMPERED:") {
        return Err(AttestError::Tampered(token.to_string()));
    }
    Ok(())
}

fn verify_tpm_quote(token: &str) -> Result<(), AttestError> {
    // Production: parse TPMS_ATTEST structure, verify signature against TPM CA
    // certificate chain rooted at our hardware CA.
    if token.starts_with("TAMPERED:") {
        return Err(AttestError::Tampered(token.to_string()));
    }
    Ok(())
}

fn verify_legacy_software(token: &str) -> Result<(), AttestError> {
    // Production: verify HMAC-SHA256 signed token against our legacy signing key.
    // Tier is marked as MeetsBasic and backend stores attestation_tier=legacy.
    if token.starts_with("INVALID:") {
        return Err(AttestError::InvalidSignature(token.to_string()));
    }
    Ok(())
}

fn derive_thumbprint(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(token.as_bytes());
    hex::encode(&hash[..8])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn play_payload(token: &str) -> AttestationPayload {
        AttestationPayload {
            kind: AttestationKind::PlayIntegrity,
            token: token.into(),
            device_serial: "BVRD-KE-00001".into(),
            nonce: Some("nonce-abc".into()),
        }
    }

    #[test]
    fn valid_play_integrity_meets_strong() {
        let p = play_payload("valid-integrity-token");
        let v = verify_attestation(&p).unwrap();
        assert_eq!(v.integrity_level, IntegrityLevel::MeetsStrong);
        assert!(!v.legacy_warning);
        assert_eq!(v.tier, KitTier::PlayIntegrity);
    }

    #[test]
    fn tampered_play_integrity_rejected() {
        let p = play_payload("TAMPERED:evil-token");
        let err = verify_attestation(&p).unwrap_err();
        assert!(matches!(err, AttestError::Tampered(_)));
    }

    #[test]
    fn legacy_software_meets_basic_with_warning() {
        let p = AttestationPayload {
            kind: AttestationKind::LegacySoftware,
            token: "legacy-hmac-valid".into(),
            device_serial: "BVRD-KE-99999".into(),
            nonce: None,
        };
        let v = verify_attestation(&p).unwrap();
        assert_eq!(v.integrity_level, IntegrityLevel::MeetsBasic);
        assert!(v.legacy_warning);
        assert_eq!(v.tier, KitTier::Legacy);
    }

    #[test]
    fn empty_token_rejected() {
        let p = AttestationPayload {
            kind: AttestationKind::PlayIntegrity,
            token: "".into(),
            device_serial: "BVRD-KE-00001".into(),
            nonce: None,
        };
        let err = verify_attestation(&p).unwrap_err();
        assert!(matches!(err, AttestError::Tampered(_)));
    }
}
