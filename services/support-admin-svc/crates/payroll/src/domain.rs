// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Domain aggregates for M55 hr-payroll (WP-5.6).
//!
//! Scale note: a production PayrollRun for 250k staff within a 4-hour SLA
//! requires parallel batch processing (e.g. rayon + chunked sqlx bulk inserts
//! with a work-stealing executor). The stubs here return immediately and are
//! intentionally simplified for the scaffolding phase.

use chrono::{DateTime, Months, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
// Documents: submission & verification (workforce-lifecycle §2)
// ---------------------------------------------------------------------------

/// Recognised upload channels for document submissions.
pub const DOCUMENT_CHANNELS: [&str; 3] = ["web", "mobile", "in_person"];

/// Doc type that triggers dual-control on rejection (design §2).
pub const DUAL_CONTROL_DOC_TYPE: &str = "national_id";

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("training gate not passed: applicant must complete training before shortlisting")]
    TrainingGateNotPassed,

    #[error("separation of duties violation: approver and initiator must be different users")]
    SodViolation,

    #[error("invalid content hash: expected a 64-char SHA-256 hex digest")]
    InvalidContentHash,

    #[error("invalid channel: expected one of web|mobile|in_person")]
    InvalidChannel,

    #[error("rejection requires a non-empty reason")]
    RejectionReasonRequired,

    #[error(
        "dual-control violation: rejecting national_id requires an X-Second-Reviewer \
         distinct from the verifier"
    )]
    DualControlViolation,
}

// ---------------------------------------------------------------------------
// Aggregates
// ---------------------------------------------------------------------------

/// A job application submitted by a prospective staff member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobApplication {
    pub id: Uuid,
    /// Human-facing application reference (e.g. `PWP-7F3K-0092`) shown to the
    /// applicant and used to track status.
    pub application_ref: String,
    pub role_code: String,
    pub county: String,
    pub applicant_name: String,
    /// SHA-256 hex of national ID number.
    pub national_id_hash: String,
    /// SHA-256 hex of the contact phone/email (never raw, DPA 2019).
    pub contact_phone_hash: Option<String>,
    pub contact_email_hash: Option<String>,
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
// Documents (workforce-lifecycle §2)
// ---------------------------------------------------------------------------

/// Role-driven required-document configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentRequirement {
    pub id: Uuid,
    pub role_code: String,
    /// Enum-like doc type: national_id|kra_pin|police_clearance|
    /// academic_cert|passport_photo|bank_letter|oath.
    pub doc_type: String,
    pub mandatory: bool,
    /// Validity window in months from verification; `None` = no expiry.
    pub validity_months: Option<u32>,
}

/// Verifier decision on a submission. Serde-tagged internally with a
/// `status` string plus variant fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Verdict {
    Pending,
    Verified {
        verifier_id: Uuid,
        verified_at: DateTime<Utc>,
    },
    Rejected {
        verifier_id: Uuid,
        reason: String,
        verified_at: DateTime<Utc>,
    },
}

/// A document submission. No raw PII — only hashes and object metadata
/// (DPA 2019); bytes live in MinIO under the applicant's tenant prefix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSubmission {
    pub id: Uuid,
    pub application_id: Uuid,
    pub doc_type: String,
    /// SHA-256 hex of the uploaded bytes.
    pub content_hash_hex: String,
    pub object_key: String,
    pub channel: String,
    pub submitted_at: DateTime<Utc>,
    /// Set when a re-upload replaces this submission; history retained.
    pub superseded_by: Option<Uuid>,
    pub verdict: Verdict,
}

/// Result of the completeness probe used by the recruitment state machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentCompleteness {
    pub complete: bool,
    pub missing: Vec<String>,
    pub expired: Vec<String>,
}

/// Default required-document matrix (design §2). Tuples are
/// `(role_code, doc_type, mandatory, validity_months)`.
pub const DEFAULT_REQUIREMENT_MATRIX: &[(&str, &str, bool, Option<u32>)] = &[
    ("po", "national_id", true, None),
    ("po", "kra_pin", true, None),
    ("po", "police_clearance", true, Some(12)),
    ("po", "academic_cert", true, None),
    ("po", "passport_photo", true, Some(6)),
    ("po", "bank_letter", false, None),
    ("po", "oath", false, None),
    ("dpo", "national_id", true, None),
    ("dpo", "kra_pin", true, None),
    ("dpo", "police_clearance", true, Some(12)),
    ("dpo", "academic_cert", true, None),
    ("dpo", "passport_photo", true, Some(6)),
    ("dpo", "bank_letter", false, None),
    ("dpo", "oath", false, None),
    ("clerk", "national_id", true, None),
    ("clerk", "kra_pin", true, None),
    ("clerk", "police_clearance", true, Some(12)),
    ("clerk", "academic_cert", false, None),
    ("clerk", "passport_photo", true, Some(6)),
    ("clerk", "bank_letter", false, None),
    ("clerk", "oath", false, None),
];

/// Deterministic requirement id derived from `(role_code, doc_type)` so the
/// seeded matrix is stable across calls and processes.
fn requirement_id(role_code: &str, doc_type: &str) -> Uuid {
    let digest = Sha256::digest(format!("{role_code}:{doc_type}").as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x70;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

fn seed_from_row(row: &(&str, &str, bool, Option<u32>)) -> DocumentRequirement {
    DocumentRequirement {
        id: requirement_id(row.0, row.1),
        role_code: row.0.to_string(),
        doc_type: row.1.to_string(),
        mandatory: row.2,
        validity_months: row.3,
    }
}

/// Seeded requirements for one role code; empty for unknown roles.
pub fn seed_requirements(role_code: &str) -> Vec<DocumentRequirement> {
    DEFAULT_REQUIREMENT_MATRIX
        .iter()
        .filter(|row| row.0 == role_code)
        .map(seed_from_row)
        .collect()
}

/// Full default matrix across all known roles.
pub fn seed_all_requirements() -> Vec<DocumentRequirement> {
    DEFAULT_REQUIREMENT_MATRIX
        .iter()
        .map(seed_from_row)
        .collect()
}

/// Validates a SHA-256 hex digest: exactly 64 hex chars decoding to 32 bytes.
pub fn is_valid_content_hash(hash: &str) -> bool {
    hash.len() == 64 && hex::decode(hash).is_ok_and(|bytes| bytes.len() == 32)
}

/// SHA-256 hex digest of a UTF-8 string. Used to fingerprint PII (national ID,
/// phone, email) before persistence; the raw value is never stored.
pub fn hash_sha256_hex(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}

/// Generate a human-facing application reference, e.g. `PWP-7F3K-0092`.
pub fn generate_application_ref() -> String {
    let s = Uuid::now_v7().simple().to_string();
    let alpha = s[..4].to_uppercase();
    let num = u32::from_str_radix(&s[4..8], 16).unwrap_or(0) % 10000;
    format!("PWP-{alpha}-{num:04}")
}

/// Returns `true` when the submission still counts as valid at `now`: it must
/// be `Verified`, and `verified_at + policy_months` must be in the future.
pub fn verify_expiry(
    submission: &DocumentSubmission,
    policy_months: u32,
    now: DateTime<Utc>,
) -> bool {
    match &submission.verdict {
        Verdict::Verified { verified_at, .. } => *verified_at + Months::new(policy_months) > now,
        Verdict::Pending | Verdict::Rejected { .. } => false,
    }
}

/// Completeness counts only the latest non-superseded submission per doc
/// type, and only when it is `Verified` and non-expired.
pub fn completeness(
    requirements: &[DocumentRequirement],
    submissions: &[DocumentSubmission],
    now: DateTime<Utc>,
) -> DocumentCompleteness {
    completeness_inner(requirements, submissions, Some(now))
}

fn completeness_inner(
    requirements: &[DocumentRequirement],
    submissions: &[DocumentSubmission],
    now: Option<DateTime<Utc>>,
) -> DocumentCompleteness {
    let mut missing = Vec::new();
    let mut expired = Vec::new();

    let active: Vec<&DocumentSubmission> = submissions
        .iter()
        .filter(|s| s.superseded_by.is_none())
        .collect();

    for req in requirements.iter().filter(|r| r.mandatory) {
        let latest = active
            .iter()
            .filter(|s| s.doc_type == req.doc_type)
            .max_by_key(|s| s.submitted_at);
        match latest {
            None => missing.push(req.doc_type.clone()),
            Some(submission) => match &submission.verdict {
                Verdict::Verified { .. } => {
                    if let (Some(now), Some(months)) = (now, req.validity_months) {
                        if !verify_expiry(submission, months, now) {
                            expired.push(req.doc_type.clone());
                        }
                    }
                }
                Verdict::Pending | Verdict::Rejected { .. } => {
                    missing.push(req.doc_type.clone());
                }
            },
        }
    }

    DocumentCompleteness {
        complete: missing.is_empty() && expired.is_empty(),
        missing,
        expired,
    }
}

/// Returns application ids whose completeness regressed purely due to expiry:
/// incomplete at `as_of`, but complete when expiry windows are ignored.
pub fn expiry_regressions(
    requirements: &[DocumentRequirement],
    submissions: &[DocumentSubmission],
    as_of: DateTime<Utc>,
) -> Vec<Uuid> {
    let mut application_ids: Vec<Uuid> = submissions.iter().map(|s| s.application_id).collect();
    application_ids.sort();
    application_ids.dedup();

    application_ids
        .into_iter()
        .filter(|application_id| {
            let own: Vec<DocumentSubmission> = submissions
                .iter()
                .filter(|s| s.application_id == *application_id)
                .cloned()
                .collect();
            let at_as_of = completeness_inner(requirements, &own, Some(as_of));
            !at_as_of.complete
                && at_as_of.missing.is_empty()
                && completeness_inner(requirements, &own, None).complete
        })
        .collect()
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
            application_ref: "PWP-TEST-0001".into(),
            role_code: "clerk".into(),
            county: "KE-022".into(),
            applicant_name: "Jane Doe".into(),
            national_id_hash: "abc123".into(),
            contact_phone_hash: None,
            contact_email_hash: None,
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
