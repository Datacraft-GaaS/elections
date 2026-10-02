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

//! Participation token and Merkle inclusion proof (WP-4.6, M14).

use serde::{Deserialize, Serialize};

/// A participation token record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationToken {
    pub token_sha256: String,
    pub station_code: String,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub inclusion_proof_ref: Option<String>,
}

/// Merkle inclusion proof for a participation token.
/// JSON-LD compatible for W3C Verifiable Credentials (FR-M14-048).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionProof {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub token_sha256: String,
    pub leaf_index: u64,
    pub tree_size: u64,
    pub audit_path: Vec<String>,
    pub root_hash: String,
}

impl InclusionProof {
    pub fn new(
        token_sha256: String,
        leaf_index: u64,
        tree_size: u64,
        audit_path: Vec<String>,
        root_hash: String,
    ) -> Self {
        Self {
            context: vec![
                "https://www.w3.org/2018/credentials/v1".into(),
                "https://transparency.eel.example/contexts/inclusion-proof/v1".into(),
            ],
            token_sha256,
            leaf_index,
            tree_size,
            audit_path,
            root_hash,
        }
    }

    /// Verify the proof locally without network.
    /// v1 stub: returns true when audit_path is non-empty and tree_size > leaf_index.
    pub fn verify(&self) -> bool {
        !self.audit_path.is_empty() && self.tree_size > self.leaf_index
    }
}
