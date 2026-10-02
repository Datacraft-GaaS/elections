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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Petition {
    pub petition_id: String,
    pub petitioner_name: String,
    pub respondent_name: String,
    pub election_id: String,
    pub constituency_code: String,
    pub filed_at: String,
    pub status: PetitionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenceBundle {
    pub bundle_id: String,
    pub petition_id: String,
    pub items: Vec<EvidenceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenceItem {
    pub item_id: String,
    pub item_type: String,
    pub description: String,
    pub document_ref: String,
    pub hash: String,
    pub submitted_at: String,
    pub submitted_by: String,
    pub verified: bool,
    pub verification_note: String,
    pub source_system: String,
    pub retention_until: String,
    pub classification: String,
    pub chain_of_custody: String,
    pub exhibit_ref: String,
    pub court_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TechnicalWitness {
    pub witness_id: String,
    pub petition_id: String,
    pub full_name: String,
    pub expertise: String,
    pub organisation: String,
    pub registered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SupplementaryRequest {
    pub request_id: String,
    pub petition_id: String,
    pub description: String,
    pub requested_by: String,
    pub requested_at: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PetitionStatus {
    #[default]
    Filed,
    Admitted,
    Hearing,
    Decided,
    Withdrawn,
}
