use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MissionStatus {
    Registered,
    Active,
    Closed,
}

impl Default for MissionStatus {
    fn default() -> Self {
        Self::Registered
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    Interim,
    Final,
}

impl Default for ReportType {
    fn default() -> Self {
        Self::Interim
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObserverMission {
    pub id: String,
    pub organisation_name: String,
    pub accreditation_ref: String,
    pub lead_contact_hash: String, // hashed - class B PII
    pub workspace_id: String,
    pub status: MissionStatus,
    pub registered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Briefing {
    pub id: String,
    pub mission_id: String,
    pub topic: String,
    pub scheduled_at: String,
    pub materials_url: Option<String>,
    pub attendance_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObservationReport {
    pub id: String,
    pub mission_id: String,
    pub report_type: ReportType,
    pub submitted_at: String,
    pub content_url: String,
    pub classified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JointStatement {
    pub id: String,
    pub participating_mission_ids: Vec<String>,
    pub published_at: String,
    pub content_url: String,
    pub signed_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegisterMissionRequest {
    pub organisation_name: String,
    pub accreditation_ref: String,
    pub lead_contact_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleBriefingRequest {
    pub topic: String,
    pub scheduled_at: String,
    pub materials_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitReportRequest {
    pub report_type: ReportType,
    pub content_url: String,
    pub classified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublishJointStatementRequest {
    pub participating_mission_ids: Vec<String>,
    pub content_url: String,
    pub signed_by: Vec<String>,
}
