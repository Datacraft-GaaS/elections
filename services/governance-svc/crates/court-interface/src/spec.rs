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
