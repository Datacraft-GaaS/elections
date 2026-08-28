use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferendumStatus {
    Draft,
    Active,
    Closed,
    Declared,
}

impl Default for ReferendumStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// M49 aggregate: Referendum
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Referendum {
    pub id: String,
    pub election_id: String,
    pub question_text: serde_json::Value, // map<lang_code, text>
    pub status: ReferendumStatus,
    /// Always 24 for constitutional referenda; stored explicitly for audit
    pub county_threshold: u8,
    /// national majority rule: 50%+1 of votes cast (integer: yes > total/2)
    pub national_majority_rule: String,
    pub scheduled_date: String,
    pub gazette_notice_id: Option<String>,
    pub created_at: String,
}

/// County-level result for a referendum (one per county, 47 total)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReferendumCountyResult {
    pub id: String,
    pub referendum_id: String,
    /// Kenya county code 001-047
    pub county_code: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub rejected_votes: u64,
    pub registered_voters: u64,
    /// provisional | certified
    pub status: String,
    pub certified_at: Option<String>,
    /// YES majority in this county: yes_votes > no_votes (integer, no FP)
    pub county_yes_majority: bool,
}

/// National aggregate result -- computed once all 47 counties report
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReferendumNationalResult {
    pub id: String,
    pub referendum_id: String,
    pub total_yes: u64,
    pub total_no: u64,
    pub total_rejected: u64,
    pub counties_reported: u8,
    pub counties_yes_majority: u8,
    /// True iff counties_yes_majority >= 24 (EC-LEG-07)
    pub meets_county_threshold: bool,
    /// True iff total_yes * 2 > (total_yes + total_no) -- integer majority of votes cast
    pub meets_national_majority: bool,
    pub computed_at: String,
}

/// Kit OTA mode bundle record -- tracks distribution and acknowledgement
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KitModeBundle {
    pub id: String,
    pub referendum_id: String,
    pub signed_bundle_hash: String,
    pub distributed_at: Option<String>,
    pub ack_count: u32,
    pub total_kits: u32,
}

/// A single answer option for a referendum question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferendumOption {
    pub label: String,
    pub code: String,
}

/// Public referendum question (M49 spec: `ReferendumQuestion`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferendumQuestion {
    pub question_id: String,
    pub text_en: String,
    pub text_sw: String,
    pub gazette_ref: String,
    pub options: Vec<ReferendumOption>,
    pub created_at: String,
}

/// Request body for registering a referendum question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferendumQuestionRequest {
    pub question_id: String,
    pub text_en: String,
    pub text_sw: String,
    pub gazette_ref: String,
    pub options: Vec<ReferendumOption>,
}

/// Per-county public result row (M49 spec: `CountyResult`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyResult {
    pub county_code: String,
    pub county_name: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub total_votes: u64,
    pub approved: bool,
}

/// National + per-county referendum results (M49 spec: `ReferendumResults`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferendumResults {
    pub question_id: String,
    pub national_yes: u64,
    pub national_no: u64,
    pub national_total: u64,
    pub county_results: Vec<CountyResult>,
    pub snapshot_at: String,
}

/// One county's approval flag in the threshold breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyBreakdown {
    pub county_code: String,
    pub approved: bool,
}

/// 24-of-47 county threshold status (M49 spec: `CountyThresholdStatus`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyThresholdStatus {
    pub counties_required: u8,
    pub counties_approved: u8,
    pub threshold_met: bool,
    pub county_breakdown: Vec<CountyBreakdown>,
    pub snapshot_at: String,
}
