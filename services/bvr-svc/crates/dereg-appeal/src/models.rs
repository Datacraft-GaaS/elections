use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeregistrationCase {
    pub id: Uuid,
    pub voter_id: Uuid,
    pub m35_event_id: Uuid,
    pub status: CaseStatus,
    pub notice_dispatched_at: Option<DateTime<Utc>>,
    /// notice_deadline = notice_dispatched_at + 30 days
    pub notice_deadline: Option<DateTime<Utc>>,
    pub appeal_id: Option<Uuid>,
    pub officer_id: Option<Uuid>,
    pub officer_recommendation: Option<String>,
    pub adjudicator_id: Option<Uuid>,
    pub adjudicator_decision: Option<AdjudicatorDecision>,
    pub adjudicator_decision_at: Option<DateTime<Utc>>,
    pub adjudicator_notes: Option<String>,
    pub data_minimisation_at: Option<DateTime<Utc>>,
    pub data_minimisation_officer: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CaseStatus {
    Noticed,
    AppealFiled,
    UnderReview,
    Adjudication,
    Overturned,
    Confirmed,
    Deregistered,
    ExpiredNoAppeal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdjudicatorDecision {
    Overturn,
    Confirm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppealRecord {
    pub id: Uuid,
    pub case_id: Uuid,
    pub intake_channel: IntakeChannel,
    /// SHA-256 of National ID
    pub appellant_id_hash: String,
    pub representative_auth_file_id: Option<Uuid>,
    pub grounds: AppealGrounds,
    pub grounds_detail: String,
    pub supporting_doc_ids: Vec<Uuid>,
    /// BCP-47 language tag
    pub language: String,
    pub filed_at: DateTime<Utc>,
    pub hearing_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntakeChannel {
    VerifierApp,
    Web,
    Ussd,
    InPerson,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppealGrounds {
    IdentityError,
    AliveAndPresent,
    DataMatchError,
    FamilyDispute,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarterlyDeregistrationReport {
    pub id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub county_breakdown: serde_json::Value,
    pub total_proposed: u64,
    pub total_noticed: u64,
    pub total_appealed: u64,
    pub total_overturned: u64,
    pub total_confirmed: u64,
    pub report_pdf_hash: Option<String>,
    pub merkle_inclusion_proof: Option<String>,
    pub commissioner_signed_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
}
