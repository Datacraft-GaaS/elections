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

//! Party-agent key registry and countersignature verification (M09).
//!
//! Anti-rigging control: an agent countersignature is only meaningful if it
//! cryptographically binds the agent's registered key to the exact form
//! content. Keys are provisioned per election via `EEL_AGENT_KEYS`
//! (`party_code:agent_id:hex_ed25519_pubkey`, comma-separated).

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::collections::HashMap;

/// Canonical bytes covered by an agent countersignature:
/// `form_id | agent_id | party_code | signed_at` (ASCII, pipe-separated).
pub fn countersign_message(
    form_id: uuid::Uuid,
    agent_id: &str,
    party_code: &str,
    signed_at: &str,
) -> Vec<u8> {
    format!("{form_id}|{agent_id}|{party_code}|{signed_at}").into_bytes()
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentKey {
    pub party_code: String,
    pub agent_id: String,
    pub verifying: VerifyingKey,
}

/// Parse the `EEL_AGENT_KEYS` registry. Returns errors listing the offending
/// entry so provisioning mistakes fail loudly at startup, not at tally time.
pub fn parse_registry(spec: &str) -> Result<Vec<AgentKey>, String> {
    let mut out = Vec::new();
    for entry in spec.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let parts: Vec<&str> = entry.split(':').collect();
        if parts.len() != 3 {
            return Err(format!(
                "bad EEL_AGENT_KEYS entry {entry:?} (want party:agent:hexkey)"
            ));
        }
        let key_bytes =
            hex::decode(parts[2]).map_err(|e| format!("bad key hex in {entry:?}: {e}"))?;
        let key_arr: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| format!("key must be 32 bytes in {entry:?}"))?;
        let verifying = VerifyingKey::from_bytes(&key_arr)
            .map_err(|e| format!("bad ed25519 key in {entry:?}: {e}"))?;
        out.push(AgentKey {
            party_code: parts[0].to_string(),
            agent_id: parts[1].to_string(),
            verifying,
        });
    }
    Ok(out)
}

pub fn lookup<'a>(
    registry: &'a [AgentKey],
    agent_id: &str,
    party_code: &str,
) -> Option<&'a AgentKey> {
    registry
        .iter()
        .find(|k| k.agent_id == agent_id && k.party_code == party_code)
}

/// Verify a hex-encoded Ed25519 countersignature over the canonical message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountersignError {
    UnknownAgent,
    BadSignatureFormat,
    InvalidSignature,
}

pub fn verify_countersignature(
    registry: &[AgentKey],
    form_id: uuid::Uuid,
    agent_id: &str,
    party_code: &str,
    signed_at: &str,
    signature_hex: &str,
) -> Result<(), CountersignError> {
    let key = lookup(registry, agent_id, party_code).ok_or(CountersignError::UnknownAgent)?;
    let sig_bytes =
        hex::decode(signature_hex.trim()).map_err(|_| CountersignError::BadSignatureFormat)?;
    let sig =
        Signature::from_slice(&sig_bytes).map_err(|_| CountersignError::BadSignatureFormat)?;
    key.verifying
        .verify(
            &countersign_message(form_id, agent_id, party_code, signed_at),
            &sig,
        )
        .map_err(|_| CountersignError::InvalidSignature)
}

/// Registry handle built once at startup from the environment.
#[derive(Debug, Clone, Default)]
pub struct AgentRegistry(HashMap<(String, String), VerifyingKey>);

impl AgentRegistry {
    pub fn from_env() -> Self {
        let spec = std::env::var("EEL_AGENT_KEYS").unwrap_or_default();
        Self::from_spec(&spec)
    }

    pub fn from_spec(spec: &str) -> Self {
        let mut map = HashMap::new();
        match parse_registry(spec) {
            Ok(keys) => {
                for k in keys {
                    map.insert((k.party_code, k.agent_id), k.verifying);
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "EEL_AGENT_KEYS malformed; NO countersignatures will be accepted")
            }
        }
        if map.is_empty() {
            tracing::warn!("EEL_AGENT_KEYS empty; all countersign requests will be rejected");
        }
        Self(map)
    }

    pub fn verify(
        &self,
        form_id: uuid::Uuid,
        agent_id: &str,
        party_code: &str,
        signed_at: &str,
        signature_hex: &str,
    ) -> Result<(), CountersignError> {
        let key = self
            .0
            .get(&(party_code.to_string(), agent_id.to_string()))
            .ok_or(CountersignError::UnknownAgent)?;
        let sig_bytes =
            hex::decode(signature_hex.trim()).map_err(|_| CountersignError::BadSignatureFormat)?;
        let sig =
            Signature::from_slice(&sig_bytes).map_err(|_| CountersignError::BadSignatureFormat)?;
        key.verify(
            &countersign_message(form_id, agent_id, party_code, signed_at),
            &sig,
        )
        .map_err(|_| CountersignError::InvalidSignature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;

    fn test_key() -> (ed25519_dalek::SigningKey, String) {
        let sk = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        let pk = hex::encode(sk.verifying_key().as_bytes());
        (sk, pk)
    }

    #[test]
    fn round_trip_accepts_valid_signature() {
        let (sk, pk) = test_key();
        let reg_spec = format!("WDM:agent-7:{pk}");
        let registry = AgentRegistry::from_spec(&reg_spec);
        let form = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        let msg = countersign_message(form, "agent-7", "WDM", "2027-08-09T18:00:00Z");
        let sig = hex::encode(sk.sign(&msg).to_bytes());
        assert!(registry
            .verify(form, "agent-7", "WDM", "2027-08-09T18:00:00Z", &sig)
            .is_ok());
    }

    #[test]
    fn rejects_wrong_form_or_agent_or_party() {
        let (sk, pk) = test_key();
        let registry = AgentRegistry::from_spec(&format!("WDM:agent-7:{pk}"));
        let form = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        let other_form = uuid::uuid!("f47ac10b-58cc-4372-a567-0e02b2c3d479");
        let msg = countersign_message(form, "agent-7", "WDM", "2027-08-09T18:00:00Z");
        let sig = hex::encode(sk.sign(&msg).to_bytes());
        assert_eq!(
            registry.verify(other_form, "agent-7", "WDM", "2027-08-09T18:00:00Z", &sig),
            Err(CountersignError::InvalidSignature)
        );
        assert_eq!(
            registry.verify(form, "agent-9", "WDM", "2027-08-09T18:00:00Z", &sig),
            Err(CountersignError::UnknownAgent)
        );
        assert_eq!(
            registry.verify(form, "agent-7", "ODM", "2027-08-09T18:00:00Z", &sig),
            Err(CountersignError::UnknownAgent)
        );
        assert_eq!(
            registry.verify(form, "agent-7", "WDM", "2027-08-09T18:00:01Z", &sig),
            Err(CountersignError::InvalidSignature)
        );
    }

    #[test]
    fn parse_registry_rejects_garbage() {
        assert!(parse_registry("nonsense").is_err());
        assert!(parse_registry("").unwrap().is_empty());
    }
}
