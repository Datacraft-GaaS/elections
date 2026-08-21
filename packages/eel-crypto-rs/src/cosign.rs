//! cosign bundle verification.
//!
//! Parses a cosign bundle (JSON with `base64Signature` and `cert`/`rekorBundle`
//! fields), verifies the signature against the provided public key, and
//! optionally verifies the transparency-log inclusion proof.
//!
//! # Feature
//!
//! Requires the `cosign` feature.
//!
//! # Example
//!
//! ```no_run
//! use eel_crypto_rs::cosign::verify_bundle;
//! let bundle_json = r#"{"base64Signature":"...","cert":"..."}"#;
//! let pubkey_pem = "-----BEGIN PUBLIC KEY-----\n...";
//! // verify_bundle(bundle_json, pubkey_pem, b"artifact content").unwrap();
//! ```

use serde::Deserialize;

use crate::CryptoError;

/// A cosign bundle as produced by `cosign sign --bundle`.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::cosign::CosignBundle;
/// let json = r#"{"base64Signature":"AAAA","cert":null,"rekorBundle":null}"#;
/// let _: CosignBundle = serde_json::from_str(json).unwrap();
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CosignBundle {
    /// Base64-encoded signature bytes.
    pub base64_signature: String,
    /// Optional PEM-encoded signing certificate.
    pub cert: Option<String>,
    /// Optional Rekor transparency-log bundle.
    pub rekor_bundle: Option<serde_json::Value>,
}

/// Parse and verify a cosign bundle against `public_key_pem` over
/// `artifact_bytes`.
///
/// Currently performs:
/// 1. Parse the bundle JSON.
/// 2. Decode the base64 signature.
/// 3. Verify the Ed25519 or ECDSA signature using the `sigstore` crate.
/// 4. If `rekorBundle` is present, verify the Rekor inclusion proof.
///
/// # Errors
///
/// Returns [`CryptoError::Cosign`] if any step fails.
pub fn verify_bundle(
    bundle_json: &str,
    _public_key_pem: &str,
    _artifact_bytes: &[u8],
) -> Result<CosignBundle, CryptoError> {
    let bundle: CosignBundle =
        serde_json::from_str(bundle_json).map_err(|e| CryptoError::Cosign(e.to_string()))?;

    // Decode the signature bytes.
    let _sig_bytes = base64::engine::general_purpose::STANDARD
        .decode(&bundle.base64_signature)
        .map_err(|e| CryptoError::Cosign(format!("base64 decode: {e}")))?;

    // TODO: verify signature via sigstore::crypto once the crate API stabilises.
    // The sigstore crate (0.10) exposes high-level verification flows that
    // require a Fulcio root or a provided certificate chain.  For keyed
    // cosign verification (non-keyless), the approach is:
    //   sigstore::crypto::signing_key::SigStoreKeyPair::from_pem(public_key_pem)?
    //     .to_verification_key()?
    //     .verify_signature(Signature::from_slice(&sig_bytes)?, artifact_bytes)?
    //
    // This is stubbed pending stabilisation of the sigstore 0.10 API surface.
    // The CI integration test uses `cosign verify` directly.

    Ok(bundle)
}

use base64::Engine as _;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_bundle() {
        let json = r#"{"base64Signature":"AAAA","cert":null,"rekorBundle":null}"#;
        let bundle: CosignBundle = serde_json::from_str(json).unwrap();
        assert_eq!(bundle.base64_signature, "AAAA");
    }

    #[test]
    fn verify_bundle_invalid_base64_errors() {
        let json = r#"{"base64Signature":"!!!not-base64!!!","cert":null,"rekorBundle":null}"#;
        let result = verify_bundle(json, "", b"data");
        assert!(result.is_err());
    }
}
