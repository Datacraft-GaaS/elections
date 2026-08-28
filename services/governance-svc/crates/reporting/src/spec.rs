//! Public contract models (governance-reporting/reports.yaml).

use serde::{Deserialize, Serialize};

/// Request body for generating a parliamentary report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRequest {
    pub period_start: String,
    pub period_end: String,
    pub report_type: String,
    pub scope: String,
}

/// Public report job view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportJob {
    pub job_id: String,
    pub period_start: String,
    pub period_end: String,
    pub report_type: String,
    pub scope: String,
    pub status: String,
    pub download_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for submitting a report to the OAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OagSubmissionRequest {
    pub report_id: String,
    pub oag_ref: String,
    pub submission_type: String,
    pub submitted_at: String,
}

/// Public OAG submission view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OagSubmission {
    pub submission_id: String,
    pub report_id: String,
    pub oag_ref: String,
    pub submission_type: String,
    pub submitted_at: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
