//! Public contract models (governance-legal-audit/audit.yaml).

use serde::{Deserialize, Serialize};

/// One step in a result-transmission chain-of-custody.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionChainStep {
    pub step: String,
    pub legal_instrument: String,
    pub officer_hash: String,
    pub timestamp: String,
    pub signature: String,
}

/// Legal basis for a transmitted form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionLegalBasis {
    pub form_id: String,
    pub transmission_chain: Vec<TransmissionChainStep>,
}

/// Request body for certifying a transmission step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionCertificationRequest {
    pub form_id: String,
    pub step: String,
    pub legal_instrument: String,
}

/// Public certification view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionCertification {
    pub certification_id: String,
    pub form_id: String,
    pub step: String,
    pub legal_instrument: String,
    pub officer_hash: String,
    pub certified_at: String,
}

/// Public legal-audit report view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalAuditReport {
    pub report_id: String,
    pub generated_at: String,
    pub scope: String,
    pub compliance_summary: serde_json::Value,
    pub findings: Vec<String>,
}
