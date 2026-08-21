use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use bvr_domain::ids::uuid7str;
use crate::error::BatchError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SarStatus {
    Pending,
    UnderReview,
    Fulfilled,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarRequest {
    pub sar_id: String,
    pub voter_id: String,
    pub requester_national_id: String,
    pub purpose: String,
    pub status: SarStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_notes: Option<String>,
}

impl SarRequest {
    pub fn new(voter_id: String, requester_national_id: String) -> Self {
        Self {
            sar_id: uuid7str(),
            voter_id,
            requester_national_id,
            purpose: "data_portability".into(),
            status: SarStatus::Pending,
            created_at: Utc::now(),
            resolved_at: None,
            resolution_notes: None,
        }
    }

    pub fn fulfill(&mut self, notes: impl Into<String>) {
        self.status = SarStatus::Fulfilled;
        self.resolved_at = Some(Utc::now());
        self.resolution_notes = Some(notes.into());
    }

    pub fn reject(&mut self, notes: impl Into<String>) {
        self.status = SarStatus::Rejected;
        self.resolved_at = Some(Utc::now());
        self.resolution_notes = Some(notes.into());
    }
}

/// Validate that a SAR can be accepted. Returns an error if already resolved.
#[allow(dead_code)]
pub fn validate_sar(req: &SarRequest) -> Result<(), BatchError> {
    match req.status {
        SarStatus::Fulfilled | SarStatus::Rejected => {
            Err(BatchError::RowValidation("SAR already resolved".into()))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sar_lifecycle() {
        let mut req = SarRequest::new("voter-1".into(), "KE12345678".into());
        assert_eq!(req.status, SarStatus::Pending);
        req.fulfill("data exported and sent to requester");
        assert_eq!(req.status, SarStatus::Fulfilled);
        assert!(req.resolved_at.is_some());
    }

    #[test]
    fn sar_reject_sets_status() {
        let mut req = SarRequest::new("voter-2".into(), "KE87654321".into());
        req.reject("requester identity not verified");
        assert_eq!(req.status, SarStatus::Rejected);
        assert!(req.resolution_notes.is_some());
    }
}
