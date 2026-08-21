use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationStatus {
    Draft,
    SubmittedIprsVerifying,
    IprsVerified,
    QrIssued,
    Converted,
    Cancelled,
    Expired,
}

/// Pre-registration application aggregate. PII tier B.
/// No raw national ID number is ever stored; only its SHA-256 hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPRSApplication {
    pub app_id: String,
    pub id_number_hash: String,
    pub status: ApplicationStatus,
    pub qr_token_hash: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 22-field BVR subset, encrypted at rest. Raw PII never logged.
    pub biographic_fields: serde_json::Value,
    pub consent_tick: bool,
    pub consent_recorded_at: Option<DateTime<Utc>>,
}

/// Append-only audit record for the application lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub actor: String,
    pub action: String,
    pub ts_hlc: DateTime<Utc>,
    /// SHA-256 of source IP address; raw IP never stored.
    pub ip_hash: String,
    pub app_id: Option<String>,
}
