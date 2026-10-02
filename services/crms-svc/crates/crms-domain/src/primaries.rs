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

//! Party primaries oversight — M46.
//! Opt-in, result submission with Merkle anchor, dispute lifecycle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryOptIn {
    pub id: Uuid,
    pub party_id: Uuid,
    pub election_cycle: String,
    pub mou_signed_at: DateTime<Utc>,
    pub approved_by: String,
    pub approved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateTally {
    pub candidate_name: String,
    pub votes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryResult {
    pub id: Uuid,
    pub party_id: Uuid,
    pub constituency_code: String,
    pub position: String,
    pub submitted_at: DateTime<Utc>,
    pub submitted_by: String,
    pub candidate_tallies: Vec<CandidateTally>,
    pub spoilt_ballots: u32,
    pub total_turnout: u32,
    /// SHA-256 of canonical JSON (FR-M46, Merkle anchor).
    pub result_hash: String,
    pub merkle_leaf_id: Option<Uuid>,
    pub published_at: Option<DateTime<Utc>>,
    pub amended: bool,
    pub amendment_reason: Option<String>,
}

impl PrimaryResult {
    pub fn build(
        party_id: Uuid,
        constituency_code: String,
        position: String,
        submitted_by: String,
        candidate_tallies: Vec<CandidateTally>,
        spoilt_ballots: u32,
        total_turnout: u32,
    ) -> Self {
        let canonical = serde_json::to_string(&candidate_tallies).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let result_hash = hex::encode(hasher.finalize());
        Self {
            id: Uuid::new_v4(),
            party_id,
            constituency_code,
            position,
            submitted_at: Utc::now(),
            submitted_by,
            candidate_tallies,
            spoilt_ballots,
            total_turnout,
            result_hash,
            merkle_leaf_id: None,
            published_at: None,
            amended: false,
            amendment_reason: None,
        }
    }

    pub fn publish(&mut self) {
        self.published_at = Some(Utc::now());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisputeStatus {
    Submitted,
    PdmReview,
    PpdtReferred,
    Hearing,
    Determination,
    Withdrawn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryDispute {
    pub id: Uuid,
    pub result_id: Uuid,
    pub complainant_name: String,
    pub complainant_id: String,
    pub grounds: String,
    pub evidence_urls: Vec<String>,
    pub status: DisputeStatus,
    pub ppdt_case_number: Option<String>,
    pub determination_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PrimaryDispute {
    pub fn new(
        result_id: Uuid,
        complainant_name: String,
        complainant_id: String,
        grounds: String,
        evidence_urls: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            result_id,
            complainant_name,
            complainant_id,
            grounds,
            evidence_urls,
            status: DisputeStatus::Submitted,
            ppdt_case_number: None,
            determination_url: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_trace_rs::trace;

    fn tallies() -> Vec<CandidateTally> {
        vec![
            CandidateTally {
                candidate_name: "A".into(),
                votes: 100,
            },
            CandidateTally {
                candidate_name: "B".into(),
                votes: 90,
            },
        ]
    }

    #[trace("FR-M46-011")]
    #[test]
    fn result_hash_is_deterministic_and_sensitive_to_tallies() {
        let r1 = PrimaryResult::build(
            Uuid::new_v4(),
            "019".into(),
            "MP".into(),
            "agent".into(),
            tallies(),
            2,
            200,
        );
        let r2 = PrimaryResult::build(
            Uuid::new_v4(),
            "019".into(),
            "MP".into(),
            "agent".into(),
            tallies(),
            2,
            200,
        );
        assert_eq!(r1.result_hash, r2.result_hash);
        assert_eq!(r1.result_hash.len(), 64);
        let mut changed = tallies();
        changed[0].votes = 101;
        let r3 = PrimaryResult::build(
            Uuid::new_v4(),
            "019".into(),
            "MP".into(),
            "agent".into(),
            changed,
            2,
            200,
        );
        assert_ne!(r1.result_hash, r3.result_hash);
    }

    #[trace("FR-M46-011")]
    #[test]
    fn fresh_result_has_no_merkle_leaf_and_not_amended() {
        let r = PrimaryResult::build(
            Uuid::new_v4(),
            "019".into(),
            "MP".into(),
            "agent".into(),
            tallies(),
            0,
            190,
        );
        assert!(r.merkle_leaf_id.is_none());
        assert!(!r.amended);
    }
}
