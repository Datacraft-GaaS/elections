//! Post-quantum hybrid signature on individual event leaves.
//!
//! Extends the eel-events envelope with a per-leaf signature combining
//! classical Ed25519 and ML-DSA-65 (Dilithium3) post-quantum signatures.
//! Both must verify for a leaf to be accepted.
//!
//! # Storage impact
//!
//! | Component          | Size       |
//! |--------------------|------------|
//! | Ed25519 sig        | 64 B       |
//! | ML-DSA-65 sig      | ~3,309 B   |
//! | key_id (UUID7 str) | ~36 B      |
//! | produced_at (ISO)  | ~30 B      |
//! | alg tag            | ~10 B      |
//! | **Total per leaf** | **~3.5 KB**|
//!
//! Classical-only leaves (64 B) remain verifiable via the `verify_classical`
//! helper when the `pq-leaf` feature flag is off.
//!
//! # Migration guide
//!
//! 1. Deploy new code with `pq-leaf` feature off (reads both old and new leaves).
//! 2. Enable `pq-leaf` to sign new leaves with hybrid signatures.
//! 3. Old classical-only leaves remain verifiable via `verify_classical`.
//! 4. After a full cycle, deprecate classical-only path.
//!
//! Algorithms:
//! - Ed25519: Bernstein et al., "High-speed high-security signatures", CHES 2011 / RFC 8032.
//! - ML-DSA-65: NIST FIPS 204 (2024), based on Ducas et al. Crystals-Dilithium, TCHES 2018.

use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature as Ed25519Sig, Signer, SigningKey, Verifier, VerifyingKey};
use oqs::sig::{Algorithm as SigAlg, Sig};
use serde::{Deserialize, Serialize};

use crate::error::EventError;

/// Post-quantum algorithm identifier stored in the signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PqAlg {
    /// ML-DSA-65 (Dilithium3), NIST FIPS 204.
    MlDsa65,
}

impl PqAlg {
    fn oqs_algorithm(&self) -> SigAlg {
        match self {
            PqAlg::MlDsa65 => SigAlg::Dilithium3,
        }
    }
}

/// Opaque ML-DSA-65 signing key (Dilithium3 secret key bytes).
///
/// Construct via [`MlDsaSigningKey::generate`].
pub struct MlDsaSigningKey {
    inner: oqs::sig::SecretKey,
    pub_key: oqs::sig::PublicKey,
    sig_ctx: Sig,
}

/// Opaque ML-DSA-65 verifying key (Dilithium3 public key bytes).
pub struct MlDsaVerifyingKey {
    inner: oqs::sig::PublicKey,
}

impl MlDsaSigningKey {
    /// Generate a fresh ML-DSA-65 key pair.
    pub fn generate() -> Result<(Self, MlDsaVerifyingKey), EventError> {
        let sig_ctx = Sig::new(SigAlg::Dilithium3)
            .map_err(|e| EventError::Crypto(format!("oqs init: {e}")))?;
        let (pk, sk) = sig_ctx
            .keypair()
            .map_err(|e| EventError::Crypto(format!("oqs keypair: {e}")))?;
        let vk = MlDsaVerifyingKey { inner: pk.clone() };
        Ok((
            MlDsaSigningKey {
                inner: sk,
                pub_key: pk,
                sig_ctx,
            },
            vk,
        ))
    }

    /// Return the corresponding verifying key.
    pub fn verifying_key(&self) -> MlDsaVerifyingKey {
        MlDsaVerifyingKey {
            inner: self.pub_key.clone(),
        }
    }
}

impl MlDsaVerifyingKey {
    /// Raw public key bytes (for storage / serialization).
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_ref()
    }

    /// Reconstruct from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EventError> {
        let sig_ctx = Sig::new(SigAlg::Dilithium3)
            .map_err(|e| EventError::Crypto(format!("oqs init: {e}")))?;
        let pk = sig_ctx
            .public_key_from_bytes(bytes)
            .ok_or_else(|| EventError::Crypto("invalid ML-DSA-65 public key".into()))?;
        Ok(MlDsaVerifyingKey {
            inner: pk.to_owned(),
        })
    }
}

/// Serde helpers for `[u8; 64]` as a base64 string.
mod classical_sig_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 64], ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.serialize_str(&base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            bytes,
        ))
    }

    pub fn deserialize<'de, D>(de: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(de)?;
        let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &s)
            .map_err(serde::de::Error::custom)?;
        bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("Ed25519 sig must be 64 bytes"))
    }
}

/// Per-leaf hybrid signature: Ed25519 (classical) + ML-DSA-65 (post-quantum).
///
/// Both components must verify for the leaf to be accepted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqLeafSignature {
    /// Ed25519 signature over the leaf payload (64 bytes, base64-encoded in JSON).
    #[serde(with = "classical_sig_serde")]
    pub classical_sig: [u8; 64],

    /// ML-DSA-65 signature bytes (~3,309 bytes for Dilithium3).
    pub pq_sig: Vec<u8>,

    /// Key identifier (UUID7 string) referencing the signing key pair in the
    /// key management service. Used for key rotation and audit.
    pub key_id: String,

    /// Timestamp when the signature was produced.
    pub produced_at: DateTime<Utc>,

    /// Post-quantum algorithm used for `pq_sig`.
    pub alg: PqAlg,
}

/// Error variants for leaf signature operations.
#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("Ed25519 verification failed: {0}")]
    Classical(String),

    #[error("ML-DSA-65 verification failed: {0}")]
    PostQuantum(String),

    #[error("oqs engine error: {0}")]
    Engine(String),
}

/// Sign `payload` with both Ed25519 and ML-DSA-65, returning a hybrid leaf signature.
///
/// `key_id` should be a UUID7 string identifying the key pair in the KMS.
///
/// # Algorithm citations
///
/// - Ed25519: Bernstein et al., "High-speed high-security signatures", CHES 2011 / RFC 8032.
/// - ML-DSA-65: NIST FIPS 204 (2024); Ducas et al. "CRYSTALS-Dilithium: A Lattice-Based
///   Digital Signature Scheme", IACR TCHES 2018.
pub fn sign_leaf(
    payload: &[u8],
    classical_key: &SigningKey,
    pq_key: &MlDsaSigningKey,
    key_id: &str,
) -> Result<PqLeafSignature, EventError> {
    let classical_sig: [u8; 64] = classical_key.sign(payload).to_bytes();

    let pq_sig = pq_key
        .sig_ctx
        .sign(payload, &pq_key.inner)
        .map_err(|e| EventError::Crypto(format!("ML-DSA-65 sign: {e}")))?
        .as_ref()
        .to_vec();

    Ok(PqLeafSignature {
        classical_sig,
        pq_sig,
        key_id: key_id.to_owned(),
        produced_at: Utc::now(),
        alg: PqAlg::MlDsa65,
    })
}

/// Verify a hybrid leaf signature against both classical and post-quantum keys.
///
/// Returns `Ok(true)` iff both Ed25519 and ML-DSA-65 verify successfully.
/// Returns `Err(VerifyError)` with a specific variant on any failure.
pub fn verify_leaf(
    payload: &[u8],
    sig: &PqLeafSignature,
    classical_pubkey: &VerifyingKey,
    pq_pubkey: &MlDsaVerifyingKey,
) -> Result<bool, VerifyError> {
    // Classical path.
    let ed_sig = Ed25519Sig::from_bytes(&sig.classical_sig);
    classical_pubkey
        .verify(payload, &ed_sig)
        .map_err(|e| VerifyError::Classical(e.to_string()))?;

    // Post-quantum path.
    let sig_ctx =
        Sig::new(sig.alg.oqs_algorithm()).map_err(|e| VerifyError::Engine(e.to_string()))?;
    let oqs_sig = sig_ctx
        .signature_from_bytes(&sig.pq_sig)
        .ok_or_else(|| VerifyError::PostQuantum("invalid signature bytes".into()))?;
    sig_ctx
        .verify(payload, oqs_sig, &pq_pubkey.inner)
        .map_err(|e| VerifyError::PostQuantum(e.to_string()))?;

    Ok(true)
}

/// Verify only the classical Ed25519 component.
///
/// Used for backward-compatible verification of classical-only leaves
/// produced before the `pq-leaf` feature was enabled.
pub fn verify_classical(
    payload: &[u8],
    classical_sig_bytes: &[u8; 64],
    classical_pubkey: &VerifyingKey,
) -> Result<bool, VerifyError> {
    let sig = Ed25519Sig::from_bytes(classical_sig_bytes);
    classical_pubkey
        .verify(payload, &sig)
        .map_err(|e| VerifyError::Classical(e.to_string()))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_crypto_rs::ed25519::generate_keypair;

    fn make_keys() -> (SigningKey, VerifyingKey, MlDsaSigningKey, MlDsaVerifyingKey) {
        let classical = generate_keypair();
        let classical_vk = classical.verifying_key();
        let (pq_sk, pq_vk) = MlDsaSigningKey::generate().expect("keygen");
        (classical, classical_vk, pq_sk, pq_vk)
    }

    #[test]
    fn sign_verify_round_trip() {
        let (ck, cvk, pqk, pqvk) = make_keys();
        let payload = b"tally-leaf:KE-01-0001:2027:1234";
        let sig =
            sign_leaf(payload, &ck, &pqk, "01950f8c-0000-7000-0000-000000000001").expect("sign");
        assert!(verify_leaf(payload, &sig, &cvk, &pqvk).expect("verify"));
    }

    #[test]
    fn tampered_payload_rejected() {
        let (ck, cvk, pqk, pqvk) = make_keys();
        let payload = b"original payload";
        let sig = sign_leaf(payload, &ck, &pqk, "key-001").expect("sign");
        let result = verify_leaf(b"tampered payload", &sig, &cvk, &pqvk);
        assert!(result.is_err());
    }

    #[test]
    fn wrong_classical_key_rejected() {
        let (ck, _cvk, pqk, pqvk) = make_keys();
        let payload = b"data";
        let sig = sign_leaf(payload, &ck, &pqk, "key-001").expect("sign");
        // Verify with a different classical key.
        let other_ck = generate_keypair();
        let other_cvk = other_ck.verifying_key();
        let result = verify_leaf(payload, &sig, &other_cvk, &pqvk);
        assert!(result.is_err());
    }

    #[test]
    fn wrong_pq_key_rejected() {
        let (ck, cvk, pqk, _pqvk) = make_keys();
        let payload = b"data";
        let sig = sign_leaf(payload, &ck, &pqk, "key-001").expect("sign");
        let (_other_pq_sk, other_pq_vk) = MlDsaSigningKey::generate().expect("keygen");
        let result = verify_leaf(payload, &sig, &cvk, &other_pq_vk);
        assert!(result.is_err());
    }

    #[test]
    fn classical_only_fallback_path() {
        let ck = generate_keypair();
        let cvk = ck.verifying_key();
        let payload = b"classical only leaf";
        let classical_sig: [u8; 64] = ck.sign(payload).to_bytes();
        assert!(verify_classical(payload, &classical_sig, &cvk).expect("verify classical"));
    }

    #[test]
    fn classical_fallback_tamper_rejected() {
        let ck = generate_keypair();
        let cvk = ck.verifying_key();
        let payload = b"original";
        let classical_sig: [u8; 64] = ck.sign(payload).to_bytes();
        let result = verify_classical(b"tampered", &classical_sig, &cvk);
        assert!(result.is_err());
    }

    #[test]
    fn pq_sig_alg_is_ml_dsa_65() {
        let (ck, _cvk, pqk, _pqvk) = make_keys();
        let sig = sign_leaf(b"data", &ck, &pqk, "key-001").expect("sign");
        assert_eq!(sig.alg, PqAlg::MlDsa65);
    }

    #[test]
    fn key_id_stored_in_sig() {
        let (ck, _cvk, pqk, _pqvk) = make_keys();
        let sig = sign_leaf(b"data", &ck, &pqk, "01950f8c-test-key-id").expect("sign");
        assert_eq!(sig.key_id, "01950f8c-test-key-id");
    }

    #[test]
    fn produced_at_is_utc() {
        let (ck, _cvk, pqk, _pqvk) = make_keys();
        let before = Utc::now();
        let sig = sign_leaf(b"data", &ck, &pqk, "k").expect("sign");
        let after = Utc::now();
        assert!(sig.produced_at >= before);
        assert!(sig.produced_at <= after);
    }

    #[test]
    fn classical_sig_is_64_bytes() {
        let (ck, _cvk, pqk, _pqvk) = make_keys();
        let sig = sign_leaf(b"data", &ck, &pqk, "k").expect("sign");
        assert_eq!(sig.classical_sig.len(), 64);
    }

    #[test]
    fn pq_sig_is_non_empty() {
        let (ck, _cvk, pqk, _pqvk) = make_keys();
        let sig = sign_leaf(b"data", &ck, &pqk, "k").expect("sign");
        assert!(!sig.pq_sig.is_empty());
    }

    #[test]
    fn pq_sig_size_is_approximately_3k() {
        // Dilithium3 signature: 3,293 bytes. Should be < 4,096 B.
        let (ck, _cvk, pqk, _pqvk) = make_keys();
        let sig = sign_leaf(b"data", &ck, &pqk, "k").expect("sign");
        assert!(
            sig.pq_sig.len() < 4096,
            "pq_sig too large: {} bytes",
            sig.pq_sig.len()
        );
        assert!(
            sig.pq_sig.len() > 2048,
            "pq_sig too small: {} bytes",
            sig.pq_sig.len()
        );
    }

    #[test]
    fn serializes_to_json() {
        let (ck, cvk, pqk, pqvk) = make_keys();
        let payload = b"json test";
        let sig = sign_leaf(payload, &ck, &pqk, "k").expect("sign");
        let json = serde_json::to_string(&sig).expect("serialize");
        let sig2: PqLeafSignature = serde_json::from_str(&json).expect("deserialize");
        assert!(verify_leaf(payload, &sig2, &cvk, &pqvk).expect("verify"));
    }

    #[test]
    fn bitflip_in_classical_sig_rejected() {
        let (ck, cvk, pqk, pqvk) = make_keys();
        let payload = b"bitflip test";
        let mut sig = sign_leaf(payload, &ck, &pqk, "k").expect("sign");
        sig.classical_sig[0] ^= 0xFF;
        let result = verify_leaf(payload, &sig, &cvk, &pqvk);
        assert!(result.is_err());
    }

    #[test]
    fn bitflip_in_pq_sig_rejected() {
        let (ck, cvk, pqk, pqvk) = make_keys();
        let payload = b"bitflip test pq";
        let mut sig = sign_leaf(payload, &ck, &pqk, "k").expect("sign");
        let last = sig.pq_sig.len() - 1;
        sig.pq_sig[last] ^= 0xFF;
        let result = verify_leaf(payload, &sig, &cvk, &pqvk);
        assert!(result.is_err());
    }

    #[test]
    fn mldsa_verifying_key_roundtrip() {
        let (pq_sk, pq_vk) = MlDsaSigningKey::generate().expect("keygen");
        let bytes = pq_vk.as_bytes().to_vec();
        let rebuilt = MlDsaVerifyingKey::from_bytes(&bytes).expect("from_bytes");
        // Sign with sk, verify with rebuilt vk.
        let (ck, cvk, _, _) = make_keys();
        let payload = b"roundtrip";
        let sig = sign_leaf(payload, &ck, &pq_sk, "k").expect("sign");
        assert!(verify_leaf(payload, &sig, &cvk, &rebuilt).expect("verify"));
    }
}
