use serde::{Deserialize, Serialize};

/// A statutory report generated for a parliamentary chamber.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatutoryReport {
    pub report_id: String,
    pub report_type: ReportType,
    pub status: ReportStatus,
    /// The reporting period, as supplied by the caller.
    pub period: String,
    /// The digest of the report body this service generated.
    ///
    /// A content hash, not a signature: no signing key or HSM is reachable from
    /// this service, so nothing here is cryptographically attributable to an
    /// officer. See `blocking_reason` when generation could not complete.
    pub content_hash: String,
    /// When generation produced no body, why.
    pub blocking_reason: Option<String>,
    pub generated_at: String,
}

/// Which chamber a report is addressed to.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReportType {
    #[default]
    Assembly,
    Senate,
    Judiciary,
    SC,
}

/// How far along generation a report is.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReportStatus {
    #[default]
    Queued,
    Generating,
    Ready,
    Delivered,
}

impl ReportStatus {
    /// The wire value, derived from serde so it cannot drift from the enum.
    pub fn wire(&self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default()
    }

    /// Whether downstream artefacts may be opened against a report in this state.
    ///
    /// Testimony, judicial handover and SC/IT pre-registration all presuppose a
    /// report that actually exists. A queued or generating report has no body to
    /// testify about.
    pub fn allows_downstream(&self) -> bool {
        matches!(self, Self::Ready | Self::Delivered)
    }
}

/// A workspace opened to prepare parliamentary testimony.
/// A request to open a testimony workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenTestimonyWorkspaceRequest {
    pub report_id: String,
    pub assignee: String,
}

/// A request to hand a report to a court.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JudiciaryHandoverRequest {
    pub report_id: String,
    pub recipient_court: String,
}

/// A request to pre-register a report with the SC/IT committee.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScPreregistrationRequest {
    pub report_id: String,
    pub sc_it_contact: String,
}

/// A request to generate a statutory report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerateStatutoryReportRequest {
    pub period: String,
    #[serde(default)]
    pub report_type: Option<ReportType>,
}

/// A workspace opened to prepare parliamentary testimony.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestimonyWorkspace {
    pub workspace_id: String,
    pub report_id: String,
    pub assignee: String,
    pub created_at: String,
    pub status: String,
}

/// A report handed to a court.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JudiciaryHandover {
    pub handover_id: String,
    pub report_id: String,
    pub recipient_court: String,
    pub handover_ref: String,
    pub handed_over_at: String,
}

/// An SC/IT pre-registration for a report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScPreregistration {
    pub prereg_id: String,
    pub report_id: String,
    pub sc_it_contact: String,
    pub registered_at: String,
    /// Null unless a committee confirmation exists. There is no SC/IT committee
    /// endpoint in this workspace to issue one.
    pub confirmation_ref: Option<String>,
}
