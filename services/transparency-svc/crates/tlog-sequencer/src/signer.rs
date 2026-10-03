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

//! STH signing backends.
//!
//! Production signs tree heads with the `eel-sth` PKCS#11 key (CKM_ECDSA,
//! P-384) via softHSM in dev and a network HSM in production (ADR-0005).
//! When no HSM is configured, a development Ed25519 key may be supplied via
//! `EEL_STH_DEV_SIGNING_KEY` so local runs still produce *verifiable*
//! signatures; falling back silently to unsigned heads is not allowed.
//!
//! Canonical signing bytes for an STH:
//! `tree_size (u64 BE) || root_hash || cadence`.
//! Verifiers recompute these from the stored row contents.

use std::sync::Arc;

use eel_crypto_rs::pkcs11_pool::{find_key_by_role, PkcsPool, PkcsPoolConfig};
use eel_crypto_rs::{CryptoError, KeyRole};
use tracing::info;

/// Which backend produced signatures, also used as the key fingerprint
/// namespace.
// Pkcs11Signer carries a pool handle and dev carries a fixed-size key; boxing
// the larger variant would change this type's layout for a per-match cost
// with no measurable gain at two variants.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum SthSigner {
    /// PKCS#11 token (softHSM / Luna / Utimaco), `eel-sth` key role.
    Pkcs11(Pkcs11Signer),
    /// Local Ed25519 development key. Refuses to run without an explicit key.
    Dev(DevEd25519),
}

#[derive(Clone)]
pub struct Pkcs11Signer {
    pool: Arc<PkcsPool>,
}

#[derive(Debug, Clone)]
pub struct DevEd25519 {
    signing: ed25519_dalek::SigningKey,
}

impl SthSigner {
    /// Build the signer from environment configuration.
    ///
    /// - `EEL_STH_SIGNER=pkcs11` (default): open the PKCS#11 pool using
    ///   `PKCS11_MODULE_PATH`, `EEL_HSM_SLOT` (default 0) and
    ///   `EEL_HSM_PIN`. SoftHSMv2 default module paths are used when
    ///   `PKCS11_MODULE_PATH` is unset.
    /// - `EEL_STH_SIGNER=dev`: require `EEL_STH_DEV_SIGNING_KEY`
    ///   (32-byte Ed25519 seed, hex). Missing key is a hard error.
    pub fn from_env() -> Result<SthSigner, CryptoError> {
        match std::env::var("EEL_STH_SIGNER")
            .unwrap_or_else(|_| "pkcs11".to_string())
            .to_lowercase()
            .as_str()
        {
            "dev" => {
                let seed_hex = std::env::var("EEL_STH_DEV_SIGNING_KEY").map_err(|_| {
                    CryptoError::InvalidKey(
                        "EEL_STH_DEV_SIGNING_KEY must be set for EEL_STH_SIGNER=dev".into(),
                    )
                })?;
                let seed: [u8; 32] = hex::decode(seed_hex.trim())
                    .map_err(|e| CryptoError::InvalidKey(format!("seed not hex: {e}")))?
                    .try_into()
                    .map_err(|_| CryptoError::InvalidKey("seed must be 32 bytes".into()))?;
                info!("STH signing: development Ed25519 key (NOT for production)");
                Ok(SthSigner::Dev(DevEd25519 {
                    signing: ed25519_dalek::SigningKey::from_bytes(&seed),
                }))
            }
            _ => {
                let module_path = std::env::var("PKCS11_MODULE_PATH")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| default_softhsm_path());
                let slot_id = std::env::var("EEL_HSM_SLOT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let pin = std::env::var("EEL_HSM_PIN").unwrap_or_default();
                let pool = PkcsPool::open(PkcsPoolConfig {
                    module_path,
                    slot_id,
                    pin,
                    max_sessions: 4,
                })?;
                info!(
                    vendor = ?std::env::var("EEL_HSM_VENDOR").unwrap_or_else(|_| "softhsm".into()),
                    "STH signing: PKCS#11 eel-sth key"
                );
                Ok(SthSigner::Pkcs11(Pkcs11Signer {
                    pool: Arc::new(pool),
                }))
            }
        }
    }

    /// Sign the canonical STH bytes. Returns `(signature, key_fingerprint)`.
    pub fn sign_sth(
        &self,
        tree_size: u64,
        root_hash: &[u8],
        cadence: &str,
    ) -> Result<(Vec<u8>, String), CryptoError> {
        let payload = canonical_sth_bytes(tree_size, root_hash, cadence);
        match self {
            SthSigner::Pkcs11(s) => s.sign_raw(&payload),
            SthSigner::Dev(s) => s.sign_raw(&payload),
        }
    }

    /// Verify a signature against canonical STH bytes.
    pub fn verify_sth(
        &self,
        tree_size: u64,
        root_hash: &[u8],
        cadence: &str,
        signature: &[u8],
    ) -> Result<bool, CryptoError> {
        let payload = canonical_sth_bytes(tree_size, root_hash, cadence);
        match self {
            SthSigner::Pkcs11(s) => s.verify_raw(&payload, signature),
            SthSigner::Dev(s) => s.verify_raw(&payload, signature),
        }
    }

    /// Stable fingerprint of the active verification key.
    pub fn fingerprint(&self) -> Result<String, CryptoError> {
        match self {
            SthSigner::Pkcs11(s) => s.fingerprint(),
            SthSigner::Dev(s) => Ok(hex::encode(s.signing.verifying_key().as_bytes())),
        }
    }
}

/// Canonical bytes covered by every STH signature.
pub fn canonical_sth_bytes(tree_size: u64, root_hash: &[u8], cadence: &str) -> Vec<u8> {
    let mut buf = Vec::with_capacity(8 + root_hash.len() + cadence.len());
    buf.extend_from_slice(&tree_size.to_be_bytes());
    buf.extend_from_slice(root_hash);
    buf.extend_from_slice(cadence.as_bytes());
    buf
}

impl Pkcs11Signer {
    fn sign_raw(&self, payload: &[u8]) -> Result<(Vec<u8>, String), CryptoError> {
        let session = self.pool.get()?;
        let handle = find_key_by_role(session.session(), KeyRole::Sth)?;
        let sig = session
            .session()
            .sign(&cryptoki::mechanism::Mechanism::Ecdsa, handle, payload)
            .map_err(|e| CryptoError::Internal(format!("sth sign failed: {e}")))?;
        let fp = read_key_id(session.session(), handle)?;
        Ok((sig, fp))
    }

    fn verify_raw(&self, payload: &[u8], signature: &[u8]) -> Result<bool, CryptoError> {
        use cryptoki::object::{Attribute, ObjectClass};
        let session = self.pool.get()?;
        let label = KeyRole::Sth.label();
        let handles = session
            .session()
            .find_objects(&[
                Attribute::Label(label.as_bytes().to_vec()),
                Attribute::Class(ObjectClass::PUBLIC_KEY),
            ])
            .map_err(|e| CryptoError::Internal(format!("find sth pubkey: {e}")))?;
        let handle = handles
            .into_iter()
            .next()
            .ok_or_else(|| CryptoError::InvalidKey("no eel-sth public key on token".into()))?;
        match session.session().verify(
            &cryptoki::mechanism::Mechanism::Ecdsa,
            handle,
            payload,
            signature,
        ) {
            Ok(()) => Ok(true),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("Signature") || msg.contains("signature") {
                    Ok(false)
                } else {
                    Err(CryptoError::Internal(format!("sth verify: {msg}")))
                }
            }
        }
    }

    fn fingerprint(&self) -> Result<String, CryptoError> {
        use cryptoki::object::{Attribute, ObjectClass};
        let session = self.pool.get()?;
        let label = KeyRole::Sth.label();
        let handles = session
            .session()
            .find_objects(&[
                Attribute::Label(label.as_bytes().to_vec()),
                Attribute::Class(ObjectClass::PUBLIC_KEY),
            ])
            .map_err(|e| CryptoError::Internal(format!("find sth pubkey: {e}")))?;
        let handle = handles
            .into_iter()
            .next()
            .ok_or_else(|| CryptoError::InvalidKey("no eel-sth public key".into()))?;
        read_key_id(session.session(), handle)
    }
}

impl DevEd25519 {
    fn sign_raw(&self, payload: &[u8]) -> Result<(Vec<u8>, String), CryptoError> {
        use ed25519_dalek::Signer;
        let sig = self.signing.sign(payload);
        Ok((
            sig.to_bytes().to_vec(),
            hex::encode(self.signing.verifying_key().as_bytes()),
        ))
    }

    fn verify_raw(&self, payload: &[u8], signature: &[u8]) -> Result<bool, CryptoError> {
        use ed25519_dalek::{Signature, Verifier};
        let arr: [u8; 64] = signature
            .try_into()
            .map_err(|_| CryptoError::InvalidSignature)?;
        match self
            .signing
            .verifying_key()
            .verify(payload, &Signature::from_bytes(&arr))
        {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

fn read_key_id(
    session: &cryptoki::session::Session,
    handle: cryptoki::object::ObjectHandle,
) -> Result<String, CryptoError> {
    use cryptoki::object::{Attribute, AttributeType};
    let attrs = session
        .get_attributes(handle, &[AttributeType::Id])
        .map_err(|e| CryptoError::Internal(format!("get_attributes: {e}")))?;
    for attr in attrs {
        if let Attribute::Id(id) = attr {
            return Ok(hex::encode(id));
        }
    }
    Ok(format!("{handle:?}"))
}

fn default_softhsm_path() -> std::path::PathBuf {
    #[cfg(target_os = "macos")]
    return std::path::PathBuf::from("/usr/local/lib/softhsm/libsofthsm2.so");
    #[cfg(target_os = "linux")]
    return std::path::PathBuf::from("/usr/lib/softhsm/libsofthsm2.so");
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return std::path::PathBuf::from("libsofthsm2.so");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev_signer() -> SthSigner {
        SthSigner::Dev(DevEd25519 {
            signing: ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]),
        })
    }

    #[test]
    fn canonical_bytes_are_size_prefixed_and_cadence_bound() {
        let a = canonical_sth_bytes(5, &[1u8; 32], "hot");
        assert_eq!(&a[..8], &5u64.to_be_bytes());
        // Different cadence over same root/size yields different bytes.
        assert_ne!(a, canonical_sth_bytes(5, &[1u8; 32], "batch"));
    }

    #[test]
    fn dev_sign_verify_round_trip() {
        let signer = dev_signer();
        let (sig, fp) = signer.sign_sth(9, &[9u8; 32], "hot").unwrap();
        assert_eq!(sig.len(), 64);
        assert_eq!(fp.len(), 64); // ed25519 pubkey hex
        assert!(signer.verify_sth(9, &[9u8; 32], "hot", &sig).unwrap());
        assert!(!signer.verify_sth(10, &[9u8; 32], "hot", &sig).unwrap());
        assert!(!signer.verify_sth(9, &[9u8; 32], "batch", &sig).unwrap());
    }

    #[test]
    fn tampered_root_fails_verification() {
        let signer = dev_signer();
        let (sig, _) = signer.sign_sth(1, &[1u8; 32], "default").unwrap();
        assert!(!signer.verify_sth(1, &[2u8; 32], "default", &sig).unwrap());
    }
}
