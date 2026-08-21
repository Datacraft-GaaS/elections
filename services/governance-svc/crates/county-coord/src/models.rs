use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum MouStatus {
    #[default]
    Draft,
    Signed,
    Active,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PartyType {
    #[default]
    County,
    Ministry,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Mou {
    pub id: String,
    pub county_code: String,
    pub ministry_code: Option<String>,
    pub party_type: PartyType,
    pub effective_date: String,
    pub expiry_date: String,
    pub content_url: Option<String>,
    pub status: MouStatus,
    pub signed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum CountyDatasetType {
    #[default]
    Population,
    SchoolFacility,
    HealthFacility,
    RoadAccess,
    SecurityHistory,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CountyDataset {
    pub id: String,
    pub county_code: String,
    pub dataset_type: CountyDatasetType,
    pub source_system: String,
    pub approved_by: Option<String>,
    pub approval_date: Option<String>,
    pub dataset_url: Option<String>,
    pub schema_version: Option<String>,
    pub lawful_basis_code: String,
    pub purpose_code: String,
    pub retention_until: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinistryApproval {
    pub id: String,
    pub mou_id: String,
    pub ministry_ref: String,
    pub approved_by: String,
    pub approved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum IncidentSeverity {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoordinationIncident {
    pub id: String,
    pub county_code: String,
    pub incident_type: String,
    pub severity: IncidentSeverity,
    pub description: Option<String>,
    pub reported_at: String,
    pub resolved_at: Option<String>,
    pub ecc_alert_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateMouRequest {
    pub county_code: String,
    pub ministry_code: Option<String>,
    pub party_type: PartyType,
    pub effective_date: String,
    pub expiry_date: String,
    pub content_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignMouRequest {
    pub officer_id: String,
    pub signature_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitDatasetRequest {
    pub county_code: String,
    pub dataset_type: CountyDatasetType,
    pub source_system: String,
    pub dataset_url: Option<String>,
    pub schema_version: Option<String>,
    pub lawful_basis_code: String,
    pub purpose_code: String,
    pub retention_until: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateIncidentRequest {
    pub county_code: String,
    pub incident_type: String,
    pub severity: IncidentSeverity,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResolveIncidentRequest {
    pub resolution_notes: String,
}
