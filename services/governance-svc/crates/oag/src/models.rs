use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OagEngagement {
    pub engagement_id: String,
    pub scope: String,
    pub status: EngagementStatus,
    pub initiated_at: String,
    pub oag_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EngagementStatus {
    #[default]
    Scoping,
    Active,
    Responded,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenceExtract {
    pub extract_id: String,
    pub engagement_id: String,
    pub description: String,
    pub payload_hash: String,
    pub extracted_at: String,
    pub extracted_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OagResponse {
    pub response_id: String,
    pub engagement_id: String,
    pub summary: String,
    pub disposition: String,
    pub responded_at: String,
    pub oag_signatory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublishedReport {
    pub pub_report_id: String,
    pub title: String,
    pub report_type: String,
    pub published_at: String,
    pub uri: String,
    pub hash: String,
}
