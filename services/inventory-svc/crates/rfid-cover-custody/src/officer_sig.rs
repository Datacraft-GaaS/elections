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

//! Officer-signature enforcement for high-risk custody transitions.
//!
//! Anti-rigging control: transitions that move physical ballot containers
//! between accountability boundaries (`STATION_CLOSED -> SEALED_RETURN ->
//! TALLY_ARRIVED -> COUNT_COMPLETE`) must carry a verifiable Ed25519 officer
//! signature over the transition's `this_hash`. A missing or invalid
//! signature on those states is a hard insert error, not a logged warning.
//!
//! Officer verification keys are provisioned per cycle via
//! `EEL_CUSTODY_OFFICER_KEYS` (`officer_uuid:hex_pubkey`, comma-separated).

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::collections::HashMap;

/// Transitions that move custody between accountable parties.
pub const HIGH_RISK_TARGET_STATES: [&str; 4] =
    ["SEALED_RETURN", "TALLY_ARRIVED", "COUNT_COMPLETE", "STORAGE_ARCHIVE"];

pub fn is_high_risk(to_state: &str) -> bool {
    HIGH_RISK_TARGET_STATES.contains(&to_state)
}

/// Canonical bytes signed by the officer: `this_hash` bytes only -- the hash
/// already commits to the full transition content and its predecessor.
pub fn signature_payload(this_hash: &[u8]) -> Vec<u8> {
    this_hash.to_vec()
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignatureCheck {
    /// Valid signature by a registered officer key.
    Verified,
    /// Not required for this transition class.
    NotRequired,
    Missing,
    UnregisteredOfficer,
    Invalid,
}

/// Verify an officer signature over a transition.
///
/// `officer_id` must match the registry key; signatures are hex-encoded
/// Ed25519 over [`signature_payload`]. Non-high-risk transitions pass with
/// `NotRequired`.
pub fn check_transition_signature(
    registry: &HashMap<String, VerifyingKey>,
    officer_id: Option<uuid::Uuid>,
    to_state: &str,
    this_hash: &[u8],
    signature: Option<&[u8]>,
) -> SignatureCheck {
    if !is_high_risk(to_state) {
        return SignatureCheck::NotRequired;
    }
    let (officer, sig_bytes) = match (officer_id, signature) {
        (Some(o), Some(s)) if !s.is_empty() => (o, s),
        _ => return SignatureCheck::Missing,
    };
    let key = match registry.get(&officer.to_string()) {
        Some(k) => k,
        None => return SignatureCheck::UnregisteredOfficer,
    };
    let sig = match Signature::from_slice(sig_bytes) {
        Ok(s) => s,
        Err(_) => return SignatureCheck::Invalid,
    };
    match key.verify(&signature_payload(this_hash), &sig) {
        Ok(()) => SignatureCheck::Verified,
        Err(_) => SignatureCheck::Invalid,
    }
}

/// Registry built once from `EEL_CUSTODY_OFFICER_KEYS`; malformed entries are
/// rejected loudly at startup rather than failing silently at tally time.
pub fn parse_registry(spec: &str) -> Result<HashMap<String, VerifyingKey>, String> {
    let mut map = HashMap::new();
    for entry in spec.split(',').map(str::trim).filter(|e| !e.is_empty()) {
        let parts: Vec<&str> = entry.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("bad entry {entry:?} (want officer_uuid:hex_pubkey)"));
        }
        let raw = hex::decode(parts[1]).map_err(|e| format!("bad hex in {entry:?}: {e}"))?;
        let arr: [u8; 32] = raw.try_into().map_err(|_| format!("key not 32 bytes in {entry:?}"))?;
        let vk = VerifyingKey::from_bytes(&arr)
            .map_err(|e| format!("bad ed25519 key in {entry:?}: {e}"))?;
        map.insert(parts[0].to_string(), vk);
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;

    fn fixture() -> (HashMap<String, VerifyingKey>, uuid::Uuid, String) {
        let sk = ed25519_dalek::SigningKey::from_bytes(&[9u8; 32]);
        let officer = uuid::uuid!("11111111-2222-3333-4444-555555555555");
        let pk = hex::encode(sk.verifying_key().as_bytes());
        let mut reg = HashMap::new();
        reg.insert(officer.to_string(), sk.verifying_key());
        (reg, officer, pk)
    }

    #[test]
    fn low_risk_transition_needs_no_signature() {
        let (reg, _, _) = fixture();
        assert_eq!(
            check_transition_signature(&reg, None, "STATION_OPENED", &[1u8; 32], None),
            SignatureCheck::NotRequired
        );
    }

    #[test]
    fn high_risk_requires_registered_key_and_valid_sig() {
        let sk = ed25519_dalek::SigningKey::from_bytes(&[9u8; 32]);
        let (reg, officer, _) = fixture();
        let hash = [7u8; 32];
        let sig = sk.sign(&signature_payload(&hash)).to_bytes();

        assert_eq!(
            check_transition_signature(&reg, Some(officer), "TALLY_ARRIVED", &hash, Some(&sig)),
            SignatureCheck::Verified
        );
        // Missing signature / unknown officer / tampered hash all fail hard.
        assert_eq!(
            check_transition_signature(&reg, None, "TALLY_ARRIVED", &hash, None),
            SignatureCheck::Missing
        );
        let stranger = uuid::uuid!("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
        assert_eq!(
            check_transition_signature(&reg, Some(stranger), "COUNT_COMPLETE", &hash, Some(&sig)),
            SignatureCheck::UnregisteredOfficer
        );
        assert_eq!(
            check_transition_signature(&reg, Some(officer), "TALLY_ARRIVED", &[8u8; 32], Some(&sig)),
            SignatureCheck::Invalid
        );
    }

    #[test]
    fn registry_parsing_rejects_garbage() {
        assert!(parse_registry("no-colons-here").is_err());
        assert!(parse_registry("").unwrap().is_empty());
    }
}
