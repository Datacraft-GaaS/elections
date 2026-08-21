//! Domain types for rectification workflow.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The type of correction being requested.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RectificationRequestType {
    BiometricRecapture,
    DemographicUpdate,
    TemplateRemoval,
    DuplicateMerge,
}

/// Lifecycle states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RectificationState {
    Submitted,
    Triage,
    EvidencePending,
    Approved,
    Rejected,
    Completed,
}

/// Required evidence type for a request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Passport,
    NationalId,
    CourtOrder,
    OdpcReference,
    MedicalCertificate,
}

/// SLA status of a request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlaStatus {
    OnTrack,
    /// Less than 3 days remaining.
    NearBreach,
    Breached,
}

/// A rectification request record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RectificationRequest {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub request_type: RectificationRequestType,
    pub state: RectificationState,
    pub submitted_at: DateTime<Utc>,
    pub sla_deadline: DateTime<Utc>,
    pub assigned_officer: Option<Uuid>,
    pub evidence_refs: Vec<String>,
    pub decision: Option<String>,
    pub decision_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub odpc_ref: Option<String>,
    pub notes: Option<String>,
}

impl RectificationRequest {
    pub const SLA_DAYS: i64 = 30;
    pub const NEAR_BREACH_DAYS: i64 = 3;

    pub fn new(subject_id: Uuid, request_type: RectificationRequestType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            subject_id,
            request_type,
            state: RectificationState::Submitted,
            submitted_at: now,
            sla_deadline: now + chrono::Duration::days(Self::SLA_DAYS),
            assigned_officer: None,
            evidence_refs: Vec::new(),
            decision: None,
            decision_at: None,
            completed_at: None,
            odpc_ref: None,
            notes: None,
        }
    }

    /// Compute SLA status relative to now.
    pub fn sla_status(&self) -> SlaStatus {
        let remaining = self
            .sla_deadline
            .signed_duration_since(Utc::now())
            .num_days();
        if remaining < 0 {
            SlaStatus::Breached
        } else if remaining <= Self::NEAR_BREACH_DAYS {
            SlaStatus::NearBreach
        } else {
            SlaStatus::OnTrack
        }
    }
}
