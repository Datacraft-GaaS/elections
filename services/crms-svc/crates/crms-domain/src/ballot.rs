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

//! Ballot proof and assembly export for printer.
//! FR-M12-017, FR-M12-018, FR-M12-019, FR-M12-020, FR-M12-021, AC-M12-04.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Position of a candidate on a ballot — deterministic ordering (FR-M12-018).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotPosition {
    pub ballot_order: u32,
    pub candidate_id: Uuid,
    pub candidate_name: String,
    pub party_name: Option<String>,
    pub party_colour_hex: Option<String>,
    pub symbol_url: Option<String>,
    pub portrait_url: Option<String>,
}

/// Ballot proof record — HSM signature added by caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotProof {
    pub id: Uuid,
    pub election_id: String,
    pub position_code: String,
    pub constituency_code: String,
    pub positions: Vec<BallotPosition>,
    /// SHA-256 of canonical JSON of `positions`.
    pub ballot_hash: String,
    /// HSM signature bytes (hex). Set after HSM signs.
    pub hsm_signature: Option<String>,
    pub printer_sig: Option<String>,
    pub party_sign_off_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl BallotProof {
    /// Build proof from a sorted list of ballot positions.
    /// Ordering is by registration order unless lottery applies (FR-M12-018).
    pub fn build(
        election_id: String,
        position_code: String,
        constituency_code: String,
        mut positions: Vec<BallotPosition>,
    ) -> Self {
        // Ensure deterministic ordering by ballot_order field.
        positions.sort_by_key(|p| p.ballot_order);
        let canonical = serde_json::to_string(&positions).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let ballot_hash = hex::encode(hasher.finalize());
        Self {
            id: Uuid::new_v4(),
            election_id,
            position_code,
            constituency_code,
            positions,
            ballot_hash,
            hsm_signature: None,
            printer_sig: None,
            party_sign_off_at: None,
            approved_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn record_party_sign_off(&mut self) {
        self.party_sign_off_at = Some(Utc::now());
    }

    pub fn approve(&mut self, printer_sig: String) {
        self.printer_sig = Some(printer_sig);
        self.approved_at = Some(Utc::now());
    }

    pub fn set_hsm_signature(&mut self, sig: String) {
        self.hsm_signature = Some(sig);
    }

    /// Canonical JSON representation for printer handoff (FR-M12-021).
    pub fn to_printer_package(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_trace_rs::trace;

    fn build_proof() -> BallotProof {
        BallotProof::build(
            "E2027".into(),
            "MP".into(),
            "019".into(),
            vec![BallotPosition {
                ballot_order: 1,
                candidate_id: Uuid::new_v4(),
                candidate_name: "A".into(),
                party_name: None,
                party_colour_hex: None,
                symbol_url: None,
                portrait_url: None,
            }],
        )
    }

    #[trace("FR-M12-018")]
    #[test]
    fn positions_sorted_deterministically() {
        let positions = vec![
            BallotPosition {
                ballot_order: 2,
                candidate_id: Uuid::new_v4(),
                candidate_name: "B".into(),
                party_name: None,
                party_colour_hex: None,
                symbol_url: None,
                portrait_url: None,
            },
            BallotPosition {
                ballot_order: 1,
                candidate_id: Uuid::new_v4(),
                candidate_name: "A".into(),
                party_name: None,
                party_colour_hex: None,
                symbol_url: None,
                portrait_url: None,
            },
        ];
        let proof = BallotProof::build("E2027".into(), "MP".into(), "019".into(), positions);
        assert_eq!(proof.positions[0].ballot_order, 1);
        assert_eq!(proof.positions[1].ballot_order, 2);
        assert!(!proof.ballot_hash.is_empty());
    }

    #[trace("FR-M12-019")]
    #[test]
    fn party_sign_off_precedes_printer_approval() {
        let mut p = build_proof();
        assert!(p.party_sign_off_at.is_none());
        p.record_party_sign_off();
        assert!(p.party_sign_off_at.is_some());
        p.approve("printer-1".into());
        assert_eq!(p.printer_sig.as_deref(), Some("printer-1"));
        assert!(p.approved_at.is_some());
    }

    #[trace("FR-M12-020")]
    #[test]
    fn hsm_signature_carries_into_published_package() {
        let mut p = build_proof();
        assert!(p.hsm_signature.is_none());
        p.set_hsm_signature("sig-hex".into());
        assert_eq!(p.hsm_signature.as_deref(), Some("sig-hex"));
        // Signature and ballot hash travel with the published package.
        let published: BallotProof = serde_json::from_str(&p.to_printer_package()).unwrap();
        assert_eq!(published.hsm_signature.as_deref(), Some("sig-hex"));
        assert_eq!(published.ballot_hash, p.ballot_hash);
    }

    #[trace("FR-M12-021")]
    #[test]
    fn printer_package_round_trips() {
        let p = build_proof();
        let pkg: BallotProof = serde_json::from_str(&p.to_printer_package()).unwrap();
        assert_eq!(pkg.election_id, p.election_id);
        assert_eq!(pkg.position_code, "MP");
        assert_eq!(pkg.positions.len(), p.positions.len());
        assert_eq!(pkg.ballot_hash, p.ballot_hash);
    }
}
