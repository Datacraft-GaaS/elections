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

//! Public contract models (governance-court/interface.yaml).

use serde::{Deserialize, Serialize};

/// Request body for filing an election petition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetitionRecordRequest {
    pub petition_ref: String,
    pub petitioner_id_hash: String,
    pub respondent_code: String,
    pub contest_code: String,
    pub grounds: String,
    pub filed_at: String,
}

/// Public election-petition record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetitionRecord {
    pub petition_id: String,
    pub petition_ref: String,
    pub petitioner_id_hash: String,
    pub respondent_code: String,
    pub contest_code: String,
    pub grounds: String,
    pub filed_at: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for creating an evidence request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequestInput {
    pub evidence_types: Vec<String>,
    pub deadline: String,
    pub court_ref: String,
}

/// Public evidence request view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequest {
    pub request_id: String,
    pub evidence_types: Vec<String>,
    pub deadline: String,
    pub court_ref: String,
    pub package_url: Option<String>,
    pub created_at: String,
}

/// Public court judgment view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourtJudgment {
    pub outcome: String,
    pub judgment_ref: String,
    pub issued_at: String,
    pub summary: Option<String>,
}
