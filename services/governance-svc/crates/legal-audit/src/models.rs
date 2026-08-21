use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransmissionRecord {
    pub record_id: String,
    pub event_type: String,
    pub sender: String,
    pub recipient: String,
    pub transmitted_at: String,
    pub payload_hash: String,
    pub legal_basis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LegalBasisRule {
    pub rule_id: String,
    pub section_ref: String,
    pub description: String,
    pub applies_to: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceVerdict {
    pub verdict_id: String,
    pub record_id: String,
    pub rule_id: String,
    pub passed: bool,
    pub tlog_hash: String,
    pub evaluated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartySummary {
    pub party_id: String,
    pub party_name: String,
    pub total_transmissions: u32,
    pub compliant: u32,
    pub non_compliant: u32,
}
