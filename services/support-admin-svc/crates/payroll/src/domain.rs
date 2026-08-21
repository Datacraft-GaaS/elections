//! Domain aggregates for M55 hr-payroll (WP-5.6).
//!
//! Scale note: a production PayrollRun for 250k staff within a 4-hour SLA
//! requires parallel batch processing (e.g. rayon + chunked sqlx bulk inserts
//! with a work-stealing executor). The stubs here return immediately and are
//! intentionally simplified for the scaffolding phase.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationStatus {
    Applied,
    Shortlisted,
    Selected,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Shortlist,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayrollStatus {
    Draft,
    Approved,
    Paid,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeductionType {
    Paye,
    Nssf,
    Nhif,
    Shif,
}

// ---------------------------------------------------------------------------
// Domain errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("training gate not passed: applicant must complete training before shortlisting")]
    TrainingGateNotPassed,

    #[error("separation of duties violation: approver and initiator must be different users")]
    SodViolation,
}

// ---------------------------------------------------------------------------
// Aggregates
// ---------------------------------------------------------------------------

/// A job application submitted by a prospective staff member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobApplication {
    pub id: Uuid,
    pub role_id: Uuid,
    pub applicant_name: String,
    /// SHA-256 hex of national ID number.
    pub national_id_hash: String,
    pub status: ApplicationStatus,
    pub applied_at: DateTime<Utc>,
    pub training_gate_passed: bool,
}

/// A hiring committee decision on a job application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortlistDecision {
    pub id: Uuid,
    pub application_id: Uuid,
    pub decided_by: Uuid,
    pub decision: Decision,
    pub reason: Option<String>,
    pub decided_at: DateTime<Utc>,
}

impl ShortlistDecision {
    /// Validates that the application has passed the training gate before
    /// a Shortlist decision is recorded.
    pub fn validate_shortlist(application: &JobApplication) -> Result<(), DomainError> {
        if !application.training_gate_passed {
            return Err(DomainError::TrainingGateNotPassed);
        }
        Ok(())
    }
}

/// Employment contract issued to staff after selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffContract {
    pub id: Uuid,
    pub staff_id: Uuid,
    pub role: String,
    pub station_id: Uuid,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    /// Gross monthly salary in Kenya Shillings (integer cents-free).
    pub gross_salary_kes: i64,
    /// SHA-256 hex of bank account number.
    pub bank_account_hash: String,
    pub deployment_ref: Option<Uuid>,
}

/// A payroll processing run covering all active staff for an election period.
///
/// Scale note: 250k staff within 4 hours requires chunked parallel payslip
/// computation. Real implementation would fan out to a task queue (e.g. Celery
/// or a Tokio JoinSet with bounded concurrency) and write payslip lines in
/// bulk-insert batches of ~1000 rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollRun {
    pub id: Uuid,
    pub election_id: Uuid,
    /// Human-readable period label, e.g. "2027-08".
    pub period: String,
    pub status: PayrollStatus,
    pub total_staff: u32,
    /// Sum of all gross salaries in KES.
    pub total_gross_kes: i64,
    pub initiated_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl PayrollRun {
    /// Enforces separation of duties: the approver must differ from the
    /// initiator. Returns DomainError::SodViolation otherwise.
    pub fn validate_approval(&self, approver: Uuid) -> Result<(), DomainError> {
        if approver == self.initiated_by {
            return Err(DomainError::SodViolation);
        }
        Ok(())
    }
}

/// A single staff member's payslip line within a PayrollRun.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayslipLine {
    pub id: Uuid,
    pub payroll_run_id: Uuid,
    pub staff_id: Uuid,
    pub gross_kes: i64,
    pub paye_kes: i64,
    pub nssf_kes: i64,
    pub nhif_kes: i64,
    pub net_kes: i64,
}

/// A statutory deduction line recorded against a payslip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatutoryDeduction {
    pub id: Uuid,
    pub payroll_run_id: Uuid,
    pub staff_id: Uuid,
    pub deduction_type: DeductionType,
    pub amount_kes: i64,
    /// Reference number from the relevant statutory body.
    pub reference: String,
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_application(training_passed: bool) -> JobApplication {
        JobApplication {
            id: Uuid::now_v7(),
            role_id: Uuid::now_v7(),
            applicant_name: "Jane Doe".into(),
            national_id_hash: "abc123".into(),
            status: ApplicationStatus::Applied,
            applied_at: Utc::now(),
            training_gate_passed: training_passed,
        }
    }

    #[test]
    fn shortlist_blocked_when_training_not_passed() {
        let app = sample_application(false);
        assert!(matches!(
            ShortlistDecision::validate_shortlist(&app),
            Err(DomainError::TrainingGateNotPassed)
        ));
    }

    #[test]
    fn shortlist_allowed_when_training_passed() {
        let app = sample_application(true);
        assert!(ShortlistDecision::validate_shortlist(&app).is_ok());
    }

    #[test]
    fn sod_violation_when_approver_equals_initiator() {
        let uid = Uuid::now_v7();
        let run = PayrollRun {
            id: Uuid::now_v7(),
            election_id: Uuid::now_v7(),
            period: "2027-08".into(),
            status: PayrollStatus::Draft,
            total_staff: 0,
            total_gross_kes: 0,
            initiated_by: uid,
            approved_by: None,
            started_at: Utc::now(),
            completed_at: None,
        };
        assert!(matches!(
            run.validate_approval(uid),
            Err(DomainError::SodViolation)
        ));
    }

    #[test]
    fn sod_ok_when_approver_differs() {
        let run = PayrollRun {
            id: Uuid::now_v7(),
            election_id: Uuid::now_v7(),
            period: "2027-08".into(),
            status: PayrollStatus::Draft,
            total_staff: 0,
            total_gross_kes: 0,
            initiated_by: Uuid::now_v7(),
            approved_by: None,
            started_at: Utc::now(),
            completed_at: None,
        };
        assert!(run.validate_approval(Uuid::now_v7()).is_ok());
    }
}
