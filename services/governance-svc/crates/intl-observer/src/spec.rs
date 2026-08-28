//! Public contract models (governance-intl-observer/coordination.yaml).

use serde::{Deserialize, Serialize};

/// Request body for registering an observer mission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverMissionRequest {
    pub mission_name: String,
    pub organisation: String,
    pub lead_observer_id_hash: String,
    pub country_of_origin: String,
    pub observer_count: u32,
    pub accreditation_ref: String,
}

/// Public observer mission view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverMission {
    pub mission_id: String,
    pub mission_name: String,
    pub organisation: String,
    pub lead_observer_id_hash: String,
    pub country_of_origin: String,
    pub observer_count: u32,
    pub accreditation_ref: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for submitting an observer report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverReportRequest {
    pub report_type: String,
    pub findings: String,
    pub recommendations: String,
    pub evidence_refs: Vec<String>,
}

/// Public observer report view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverReport {
    pub report_id: String,
    pub mission_id: String,
    pub report_type: String,
    pub findings: String,
    pub recommendations: String,
    pub evidence_refs: Vec<String>,
    pub created_at: String,
}
