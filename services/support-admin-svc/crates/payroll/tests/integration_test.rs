//! Integration tests for the payroll crate (WP-5.6).

use chrono::Utc;
use payroll::domain::{
    ApplicationStatus, Decision, DeductionType, DomainError, JobApplication, PayrollRun,
    PayrollStatus, ShortlistDecision,
};
use uuid::Uuid;

fn make_application(training_passed: bool) -> JobApplication {
    JobApplication {
        id: Uuid::now_v7(),
        role_id: Uuid::now_v7(),
        applicant_name: "Test Applicant".into(),
        national_id_hash: "deadbeef".into(),
        status: ApplicationStatus::Applied,
        applied_at: Utc::now(),
        training_gate_passed: training_passed,
    }
}

#[test]
fn shortlist_requires_training_gate() {
    let app = make_application(false);
    assert!(matches!(
        ShortlistDecision::validate_shortlist(&app),
        Err(DomainError::TrainingGateNotPassed)
    ));
}

#[test]
fn shortlist_passes_when_trained() {
    let app = make_application(true);
    ShortlistDecision::validate_shortlist(&app).unwrap();
}

#[test]
fn payroll_sod_enforced() {
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
    assert!(run.validate_approval(Uuid::now_v7()).is_ok());
}

#[test]
fn decision_serialises() {
    let d = Decision::Shortlist;
    let s = serde_json::to_string(&d).unwrap();
    assert_eq!(s, r#""shortlist""#);
}

#[test]
fn deduction_type_serialises() {
    let d = DeductionType::Paye;
    let s = serde_json::to_string(&d).unwrap();
    assert_eq!(s, r#""paye""#);
}
