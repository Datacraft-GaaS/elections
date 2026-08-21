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
