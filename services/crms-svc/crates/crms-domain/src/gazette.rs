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

//! Gazette generator — dual-officer sign-off, signed JSON and CSV output.
//! FR-M12-015, FR-M12-016, FR-M12-020, EC-IDN-09.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazetteEntry {
    pub candidate_id: Uuid,
    pub name: String,
    pub position: String,
    pub constituency_code: String,
    pub party_name: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gazette {
    pub id: Uuid,
    pub election_id: String,
    pub position_code: String,
    pub entries: Vec<GazetteEntry>,
    pub content_hash: String,
    pub officer_a_id: String,
    pub officer_b_id: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Gazette {
    pub fn build(
        election_id: String,
        position_code: String,
        entries: Vec<GazetteEntry>,
        officer_a_id: String,
        officer_b_id: String,
    ) -> Self {
        let canonical = serde_json::to_string(&entries).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let content_hash = hex::encode(hasher.finalize());
        Self {
            id: Uuid::new_v4(),
            election_id,
            position_code,
            entries,
            content_hash,
            officer_a_id,
            officer_b_id,
            published_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn publish(&mut self) {
        self.published_at = Some(Utc::now());
    }

    /// Emit as signed JSON string. HSM signature added by caller via eel-crypto-rs.
    pub fn to_signed_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Simple CSV representation for printer and public gazette portal.
    pub fn to_csv(&self) -> String {
        let mut out = String::from("candidate_id,name,position,constituency,party,status\n");
        for e in &self.entries {
            out.push_str(&format!(
                "{},{},{},{},{},{}\n",
                e.candidate_id,
                e.name,
                e.position,
                e.constituency_code,
                e.party_name.as_deref().unwrap_or("INDEPENDENT"),
                e.status,
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_trace_rs::trace;

    fn entry(name: &str, party: Option<String>) -> GazetteEntry {
        GazetteEntry {
            candidate_id: Uuid::new_v4(),
            name: name.into(),
            position: "MP".into(),
            constituency_code: "019".into(),
            party_name: party,
            status: "CLEARED".into(),
        }
    }

    #[trace("FR-M12-016")]
    #[test]
    fn to_csv_emits_header_and_rows_with_independent_default() {
        let g = Gazette::build(
            "E2027".into(),
            "MP".into(),
            vec![entry("A", Some("Party X".into())), entry("B", None)],
            "officer-a".into(),
            "officer-b".into(),
        );
        let csv = g.to_csv();
        let lines: Vec<&str> = csv.trim_end().split('\n').collect();
        assert_eq!(
            lines[0],
            "candidate_id,name,position,constituency,party,status"
        );
        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains("Party X"));
        assert!(lines[2].contains("INDEPENDENT"));
    }

    #[trace("FR-M12-016")]
    #[test]
    fn content_hash_is_64_hex_and_sensitive_to_entries() {
        let g1 = Gazette::build(
            "E2027".into(),
            "MP".into(),
            vec![entry("A", None)],
            "a".into(),
            "b".into(),
        );
        let g2 = Gazette::build(
            "E2027".into(),
            "MP".into(),
            vec![entry("A2", None)],
            "a".into(),
            "b".into(),
        );
        assert_eq!(g1.content_hash.len(), 64);
        assert_ne!(g1.content_hash, g2.content_hash);
    }
}
