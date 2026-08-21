use chrono::{Duration, Utc};
use dereg_appeal::models::{CaseStatus, DeregistrationCase};
use dereg_appeal::workflow::advance;
use uuid::Uuid;

fn noticed_case(deadline_offset_days: i64) -> DeregistrationCase {
    DeregistrationCase {
        id: Uuid::now_v7(),
        voter_id: Uuid::now_v7(),
        m35_event_id: Uuid::now_v7(),
        status: CaseStatus::Noticed,
        notice_dispatched_at: Some(Utc::now()),
        notice_deadline: Some(Utc::now() + Duration::days(deadline_offset_days)),
        appeal_id: None,
        officer_id: None,
        officer_recommendation: None,
        adjudicator_id: None,
        adjudicator_decision: None,
        adjudicator_decision_at: None,
        adjudicator_notes: None,
        data_minimisation_at: None,
        data_minimisation_officer: None,
        created_at: Utc::now(),
    }
}

#[test]
fn noticed_with_future_deadline_stays_noticed() {
    let mut case = noticed_case(30);
    advance(&mut case);
    assert_eq!(case.status, CaseStatus::Noticed);
}

#[test]
fn noticed_past_deadline_no_appeal_expires() {
    let mut case = noticed_case(-1);
    advance(&mut case);
    assert_eq!(case.status, CaseStatus::ExpiredNoAppeal);
}

#[test]
fn noticed_past_deadline_with_appeal_stays_noticed() {
    let mut case = noticed_case(-1);
    case.appeal_id = Some(Uuid::now_v7());
    advance(&mut case);
    assert_eq!(case.status, CaseStatus::Noticed);
}
