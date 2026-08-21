use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatutoryReport {
    pub report_id: String,
    pub report_type: ReportType,
    pub status: ReportStatus,
    pub period: String,
    pub sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ReportType {
    #[default]
    Assembly,
    Senate,
    Judiciary,
    SC,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ReportStatus {
    #[default]
    Queued,
    Generating,
    Ready,
    Delivered,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestimonyWorkspace {
    pub workspace_id: String,
    pub report_id: String,
    pub assignee: String,
    pub created_at: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JudiciaryHandover {
    pub handover_id: String,
    pub report_id: String,
    pub recipient_court: String,
    pub handover_ref: String,
    pub handed_over_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScPreregistration {
    pub prereg_id: String,
    pub report_id: String,
    pub sc_it_contact: String,
    pub registered_at: String,
    pub confirmation_ref: String,
}
