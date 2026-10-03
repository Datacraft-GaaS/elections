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

//! Append-only audit chain for reviewer actions.
//!
//! Every `ReviewerAction` is signed over its canonical JSON representation.
//! Actions form a hash chain via `prev_action_hash` (BLAKE3 of the previous
//! action's canonical JSON). Tampering with any past action breaks the chain.

use chrono::Utc;
use hex;
use serde_json;
use uuid::Uuid;

use crate::domain::{ActionType, ReviewerAction};

/// HSM-backed signer abstraction. In production, backed by eel-crypto-rs
/// PKCS#11 pool; in tests, backed by `Ed25519TestSigner`.
pub trait HsmSigner: Send + Sync {
    fn sign(&self, data: &[u8]) -> Vec<u8>;
    fn public_key_hex(&self) -> String;
}

/// Build canonical JSON for a pending action (excludes signature fields).
#[allow(clippy::too_many_arguments)]
fn canonical_json_for(
    id: &Uuid,
    case_id: &Uuid,
    reviewer_id: &Uuid,
    action_type: &ActionType,
    comment: &Option<String>,
    evidence_added: &[Uuid],
    taken_at: &chrono::DateTime<Utc>,
    prev_action_hash: &Option<String>,
) -> String {
    // Deterministic field ordering for signing.
    let evidence: Vec<String> = evidence_added.iter().map(|u| u.to_string()).collect();
    let mut obj = serde_json::json!({
        "id": id.to_string(),
        "case_id": case_id.to_string(),
        "reviewer_id": reviewer_id.to_string(),
        "action_type": action_type.as_str(),
        "evidence_added": evidence,
        "taken_at": taken_at.to_rfc3339(),
    });
    if let Some(c) = comment {
        obj["comment"] = serde_json::Value::String(c.clone());
    }
    if let Some(h) = prev_action_hash {
        obj["prev_action_hash"] = serde_json::Value::String(h.clone());
    }
    // Sort keys for determinism.
    canonical_sort(&obj)
}

/// Recursively sort JSON object keys for canonical form.
fn canonical_sort(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted: Vec<_> = map.iter().collect();
            sorted.sort_by_key(|(k, _)| k.as_str());
            let inner: Vec<String> = sorted
                .iter()
                .map(|(k, v)| format!("{:?}:{}", k, canonical_sort(v)))
                .collect();
            format!("{{{}}}", inner.join(","))
        }
        serde_json::Value::Array(arr) => {
            let inner: Vec<String> = arr.iter().map(canonical_sort).collect();
            format!("[{}]", inner.join(","))
        }
        other => other.to_string(),
    }
}

/// BLAKE3 hash of canonical JSON bytes, returned as lowercase hex.
pub fn hash_canonical(canonical: &str) -> String {
    let digest = blake3::hash(canonical.as_bytes());
    hex::encode(digest.as_bytes())
}

/// Verify the integrity of an action's signature.
///
/// Returns true if the ed25519 signature over `canonical_json` verifies
/// against `public_key_hex`.
pub fn verify_action_signature(action: &ReviewerAction) -> Result<(), AuditError> {
    use eel_crypto_rs::ed25519::{verify, Ed25519VerifyingKey};

    let key_bytes =
        hex::decode(&action.public_key_hex).map_err(|_| AuditError::InvalidPublicKey)?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| AuditError::InvalidPublicKey)?;
    let verifying_key =
        Ed25519VerifyingKey::from_bytes(&key_array).map_err(|_| AuditError::InvalidPublicKey)?;

    let sig_bytes: [u8; 64] = action
        .hsm_signature
        .as_slice()
        .try_into()
        .map_err(|_| AuditError::InvalidSignature)?;
    let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);

    verify(&verifying_key, action.canonical_json.as_bytes(), &sig)
        .map_err(|_| AuditError::InvalidSignature)
}

/// Verify the entire hash chain for a sequence of actions (ordered by taken_at).
///
/// Returns `Err(AuditError::ChainBroken)` at the first broken link.
pub fn verify_chain(actions: &[ReviewerAction]) -> Result<(), AuditError> {
    let mut expected_hash: Option<String> = None;

    for action in actions {
        // Check prev_action_hash matches what we computed.
        if action.prev_action_hash != expected_hash {
            return Err(AuditError::ChainBroken {
                action_id: action.id,
                expected: expected_hash,
                found: action.prev_action_hash.clone(),
            });
        }
        // Advance expected hash to current action's canonical hash.
        expected_hash = Some(hash_canonical(&action.canonical_json));
    }
    Ok(())
}

/// Create a signed `ReviewerAction`, chaining from the previous action's hash.
pub fn create_action(
    case_id: Uuid,
    reviewer_id: Uuid,
    action_type: ActionType,
    comment: Option<String>,
    evidence_added: Vec<Uuid>,
    prev_action: Option<&ReviewerAction>,
    signer: &dyn HsmSigner,
) -> ReviewerAction {
    let id = Uuid::now_v7();
    let taken_at = Utc::now();
    let prev_action_hash = prev_action.map(|a| hash_canonical(&a.canonical_json));

    let canonical = canonical_json_for(
        &id,
        &case_id,
        &reviewer_id,
        &action_type,
        &comment,
        &evidence_added,
        &taken_at,
        &prev_action_hash,
    );

    let signature = signer.sign(canonical.as_bytes());
    let public_key_hex = signer.public_key_hex();

    ReviewerAction {
        id,
        case_id,
        reviewer_id,
        action_type,
        comment,
        evidence_added,
        taken_at,
        hsm_signature: signature,
        public_key_hex,
        prev_action_hash,
        canonical_json: canonical,
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum AuditError {
    #[error("invalid public key encoding")]
    InvalidPublicKey,
    #[error("signature verification failed")]
    InvalidSignature,
    #[error("hash chain broken at action {action_id}: expected {expected:?}, found {found:?}")]
    ChainBroken {
        action_id: Uuid,
        expected: Option<String>,
        found: Option<String>,
    },
}

/// Ed25519 test double that uses an ephemeral key pair.
///
/// Unconditionally public so integration tests (which compile the crate
/// without `cfg(test)`) can import it; inert in production binaries.
pub mod test_utils {
    use super::HsmSigner;
    use eel_crypto_rs::ed25519::{generate_keypair, sign, Ed25519SigningKey};

    pub struct Ed25519TestSigner {
        key: Ed25519SigningKey,
    }

    impl Default for Ed25519TestSigner {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Ed25519TestSigner {
        pub fn new() -> Self {
            Self {
                key: generate_keypair(),
            }
        }
    }

    impl HsmSigner for Ed25519TestSigner {
        fn sign(&self, data: &[u8]) -> Vec<u8> {
            sign(&self.key, data).to_bytes().to_vec()
        }
        fn public_key_hex(&self) -> String {
            hex::encode(self.key.verifying_key().as_bytes())
        }
    }
}
