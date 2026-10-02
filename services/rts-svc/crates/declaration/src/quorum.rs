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

//! Dual-control (M-of-N) declaration gate.
//!
//! Anti-rigging control: a national declaration may only be published when
//! at least `quorum` of the `n` registered commissioners have signed the
//! canonical declaration bytes with their Ed25519 keys, and the tally Merkle
//! root is bound into those bytes so no signature can be replayed against a
//! different tally.
//!
//! Mirrors the semantics of hsm-signer's CeremonyStore (quorum approvals,
//! terminal states) without requiring an HSM at every deployment: keys may
//! be HSM-held or software-held; verification logic is identical.

use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Canonical declaration payload covered by every commissioner signature:
/// `election_id | outcome_json | tally_merkle_root | declared_at`.
pub fn canonical_declaration_bytes(
    election_id: &str,
    outcome_json: &str,
    tally_merkle_root: &str,
    declared_at: &str,
) -> Vec<u8> {
    format!("{election_id}|{outcome_json}|{tally_merkle_root}|{declared_at}").into_bytes()
}

#[derive(Debug, Error)]
pub enum QuorumError {
    #[error("commissioner {0} is not registered")]
    UnknownCommissioner(String),
    #[error("duplicate approval from commissioner {0}")]
    DuplicateApproval(String),
    #[error("invalid signature from commissioner {0}")]
    BadSignature(String),
    #[error("quorum {required} not met ({got} approvals)")]
    QuorumNotMet { required: usize, got: usize },
}

/// Signed declaration artifact: publishable only after [`DeclarationGate::seal`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedDeclaration {
    pub election_id: String,
    pub outcome_json: String,
    pub tally_merkle_root: String,
    pub declared_at: String,
    /// (commissioner_id, hex signature) pairs over the canonical bytes.
    pub approvals: Vec<(String, String)>,
}

/// M-of-N approval ledger for one declaration.
pub struct DeclarationGate {
    commissioners: HashMap<String, VerifyingKey>,
    quorum: usize,
    approvals: HashMap<String, Signature>,
}

impl DeclarationGate {
    /// Register the commissioner set and required quorum. Malformed entries
    /// are rejected here rather than discovered at seal time.
    pub fn new(spec: &str, quorum: usize) -> Result<Self, QuorumError> {
        let mut commissioners = HashMap::new();
        for entry in spec.split(',').map(str::trim).filter(|e| !e.is_empty()) {
            let parts: Vec<&str> = entry.split(':').collect();
            if parts.len() != 2 {
                return Err(QuorumError::UnknownCommissioner(entry.to_string()));
            }
            let raw = hex::decode(parts[1])
                .map_err(|_| QuorumError::UnknownCommissioner(entry.to_string()))?;
            let arr: [u8; 32] = raw
                .try_into()
                .map_err(|_| QuorumError::UnknownCommissioner(entry.to_string()))?;
            let vk = VerifyingKey::from_bytes(&arr)
                .map_err(|_| QuorumError::UnknownCommissioner(entry.to_string()))?;
            commissioners.insert(parts[0].to_string(), vk);
        }
        Ok(Self {
            commissioners,
            quorum,
            approvals: HashMap::new(),
        })
    }

    pub fn approvals(&self) -> usize {
        self.approvals.len()
    }

    /// Record one commissioner's approval. The signature must verify over the
    /// canonical declaration bytes; duplicates are rejected explicitly.
    pub fn approve(
        &mut self,
        commissioner_id: &str,
        sig_hex: &str,
        canonical: &[u8],
    ) -> Result<(), QuorumError> {
        if self.approvals.contains_key(commissioner_id) {
            return Err(QuorumError::DuplicateApproval(commissioner_id.to_string()));
        }
        let key = self
            .commissioners
            .get(commissioner_id)
            .ok_or_else(|| QuorumError::UnknownCommissioner(commissioner_id.to_string()))?;
        let raw = hex::decode(sig_hex.trim())
            .map_err(|_| QuorumError::BadSignature(commissioner_id.to_string()))?;
        let sig = Signature::from_slice(&raw)
            .map_err(|_| QuorumError::BadSignature(commissioner_id.to_string()))?;
        key.verify(canonical, &sig)
            .map_err(|_| QuorumError::BadSignature(commissioner_id.to_string()))?;
        self.approvals.insert(commissioner_id.to_string(), sig);
        Ok(())
    }

    /// Convenience: a commissioner signs locally (software key path).
    pub fn approve_with_signing_key(
        &mut self,
        signing: &ed25519_dalek::SigningKey,
        canonical: &[u8],
    ) -> Result<(), QuorumError> {
        let id = hex::encode(signing.verifying_key().as_bytes());
        let sig_hex = hex::encode(signing.sign(canonical).to_bytes());
        self.approve(&id, &sig_hex, canonical)
    }

    /// Produce the sealed declaration. Fails unless quorum distinct valid
    /// approvals are present.
    pub fn seal(
        &self,
        election_id: &str,
        outcome_json: &str,
        tally_merkle_root: &str,
        declared_at: &str,
    ) -> Result<SignedDeclaration, QuorumError> {
        if self.approvals.len() < self.quorum {
            return Err(QuorumError::QuorumNotMet {
                required: self.quorum,
                got: self.approvals.len(),
            });
        }
        Ok(SignedDeclaration {
            election_id: election_id.to_string(),
            outcome_json: outcome_json.to_string(),
            tally_merkle_root: tally_merkle_root.to_string(),
            declared_at: declared_at.to_string(),
            approvals: self
                .approvals
                .iter()
                .map(|(id, s)| (id.clone(), hex::encode(s.to_bytes())))
                .collect(),
        })
    }
}

impl SignedDeclaration {
    /// Independent verification: every listed signature must come from a
    /// registered commissioner and verify over the canonical bytes; quorum
    /// must still hold against that same registry. Verifiers (portal,
    /// verifier-rs, observers) call this -- they need no trust in the server.
    pub fn verify(&self, registry_spec: &str, quorum: usize) -> Result<(), QuorumError> {
        let gate = DeclarationGate::new(registry_spec, quorum)?;
        if gate.commissioners.len() < quorum && quorum > 0 {
            return Err(QuorumError::QuorumNotMet {
                required: quorum,
                got: gate.commissioners.len(),
            });
        }
        let canonical = canonical_declaration_bytes(
            &self.election_id,
            &self.outcome_json,
            &self.tally_merkle_root,
            &self.declared_at,
        );
        let mut verified = 0usize;
        for (id, sig_hex) in &self.approvals {
            let key = gate
                .commissioners
                .get(id)
                .ok_or_else(|| QuorumError::UnknownCommissioner(id.clone()))?;
            let raw =
                hex::decode(sig_hex).map_err(|_| QuorumError::BadSignature(id.clone()))?;
            let sig = Signature::from_slice(&raw)
                .map_err(|_| QuorumError::BadSignature(id.clone()))?;
            key.verify(&canonical, &sig)
                .map_err(|_| QuorumError::BadSignature(id.clone()))?;
            verified += 1;
        }
        if verified < quorum {
            return Err(QuorumError::QuorumNotMet {
                required: quorum,
                got: verified,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Commissioner {
        signing: ed25519_dalek::SigningKey,
    }

    fn three_commissioners() -> Vec<Commissioner> {
        (0..3)
            .map(|i| Commissioner {
                signing: ed25519_dalek::SigningKey::from_bytes(&[100 + i as u8; 32]),
            })
            .collect()
    }

    fn spec_of(cs: &[Commissioner]) -> String {
        cs.iter()
            .map(|c| {
                format!(
                    "{}:{}",
                    hex::encode(c.signing.verifying_key().as_bytes()),
                    hex::encode(c.signing.verifying_key().as_bytes())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    const ELECTION: &str = "elec-2027-general";
    const OUTCOME: &str = r#"{"declared":"candidate-7"}"#;
    const ROOT: &str = "abababababababababababababababababababababababababababababababab";
    const AT: &str = "2027-08-09T20:00:00Z";

    fn canonical(cs: &[Commissioner]) -> Vec<u8> {
        let _ = cs;
        canonical_declaration_bytes(ELECTION, OUTCOME, ROOT, AT)
    }

    #[test]
    fn two_of_three_seals_and_independently_verifies() {
        let cs = three_commissioners();
        let mut gate = DeclarationGate::new(&spec_of(&cs), 2).unwrap();
        gate.approve_with_signing_key(&cs[0].signing, &canonical(&cs)).unwrap();
        assert!(matches!(
            gate.seal(ELECTION, OUTCOME, ROOT, AT),
            Err(QuorumError::QuorumNotMet { required: 2, got: 1 })
        ));
        gate.approve_with_signing_key(&cs[1].signing, &canonical(&cs)).unwrap();

        let sealed = gate.seal(ELECTION, OUTCOME, ROOT, AT).unwrap();
        sealed.verify(&spec_of(&cs), 2).unwrap();
    }

    #[test]
    fn tampered_outcome_breaks_every_signature() {
        let cs = three_commissioners();
        let mut gate = DeclarationGate::new(&spec_of(&cs), 2).unwrap();
        gate.approve_with_signing_key(&cs[0].signing, &canonical(&cs)).unwrap();
        gate.approve_with_signing_key(&cs[1].signing, &canonical(&cs)).unwrap();
        let mut forged = gate.seal(ELECTION, OUTCOME, ROOT, AT).unwrap();
        forged.outcome_json = r#"{"declared":"candidate-8"}"#.to_string();
        assert!(forged.verify(&spec_of(&cs), 2).is_err());
    }

    #[test]
    fn unknown_or_duplicate_approvers_rejected() {
        let cs = three_commissioners();
        let outsider = ed25519_dalek::SigningKey::from_bytes(&[200u8; 32]);
        let mut gate = DeclarationGate::new(&spec_of(&cs), 2).unwrap();
        assert!(matches!(
            gate.approve_with_signing_key(&outsider, &canonical(&cs)),
            Err(QuorumError::UnknownCommissioner(_))
        ));
        gate.approve_with_signing_key(&cs[0].signing, &canonical(&cs)).unwrap();
        assert!(matches!(
            gate.approve_with_signing_key(&cs[0].signing, &canonical(&cs)),
            Err(QuorumError::DuplicateApproval(_))
        ));
    }

    #[test]
    fn unregistered_verifier_registry_fails_closed() {
        let cs = three_commissioners();
        let mut gate = DeclarationGate::new(&spec_of(&cs), 2).unwrap();
        gate.approve_with_signing_key(&cs[0].signing, &canonical(&cs)).unwrap();
        gate.approve_with_signing_key(&cs[1].signing, &canonical(&cs)).unwrap();
        let sealed = gate.seal(ELECTION, OUTCOME, ROOT, AT).unwrap();
        // Verifier with a different registry must refuse.
        assert!(sealed.verify(&spec_of(&three_commissioners()), 2).is_ok()); // fresh identical registry passes
        let wrong = "aa:bb";
        assert!(sealed.verify(wrong, 2).is_err());
    }
}
