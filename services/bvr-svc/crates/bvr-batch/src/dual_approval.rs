use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::error::BatchError;
use bvr_domain::ids::uuid7str;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    AwaitingFirst,
    AwaitingSecond,
    Approved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualApprovalWorkflow {
    pub workflow_id: String,
    /// Resource being approved, e.g. "export/<scope>"
    pub resource: String,
    pub first_approver: Option<String>,
    pub second_approver: Option<String>,
    pub status: ApprovalStatus,
    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
}

impl DualApprovalWorkflow {
    pub fn new(resource: impl Into<String>) -> Self {
        Self {
            workflow_id: uuid7str(),
            resource: resource.into(),
            first_approver: None,
            second_approver: None,
            status: ApprovalStatus::AwaitingFirst,
            created_at: Utc::now(),
            approved_at: None,
        }
    }

    pub fn record_approval(&mut self, approver: String) -> Result<(), BatchError> {
        match &self.status {
            ApprovalStatus::AwaitingFirst => {
                self.first_approver = Some(approver);
                self.status = ApprovalStatus::AwaitingSecond;
                Ok(())
            }
            ApprovalStatus::AwaitingSecond => {
                if self.first_approver.as_deref() == Some(approver.as_str()) {
                    return Err(BatchError::DuplicateApproval(approver));
                }
                self.second_approver = Some(approver);
                self.status = ApprovalStatus::Approved;
                self.approved_at = Some(Utc::now());
                Ok(())
            }
            ApprovalStatus::Approved => Err(BatchError::DuplicateApproval(approver)),
        }
    }

    pub fn is_approved(&self) -> bool {
        self.status == ApprovalStatus::Approved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_person_rule_enforced() {
        let mut wf = DualApprovalWorkflow::new("export/2027-general");
        assert!(!wf.is_approved());
        wf.record_approval("alice".into()).unwrap();
        // Same person cannot give second approval
        assert!(wf.record_approval("alice".into()).is_err());
        wf.record_approval("bob".into()).unwrap();
        assert!(wf.is_approved());
    }

    #[test]
    fn third_approval_on_approved_workflow_errors() {
        let mut wf = DualApprovalWorkflow::new("export/2027-general");
        wf.record_approval("alice".into()).unwrap();
        wf.record_approval("bob".into()).unwrap();
        assert!(wf.record_approval("charlie".into()).is_err());
    }
}
