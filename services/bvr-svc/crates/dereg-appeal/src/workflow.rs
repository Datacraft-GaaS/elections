//! 30+14-day deregistration workflow enforcer.
//!
//! Day 0: notice dispatched (notice_deadline = +30 days).
//! Days 1-30: voter may file appeal (opens AppealRecord).
//! Days 31-44: appeal review window (14 days).
//! Day 45+: if no appeal and no overturn, case moves to Deregistered.

use crate::models::{CaseStatus, DeregistrationCase};
use chrono::Utc;

pub fn advance(case: &mut DeregistrationCase) {
    let now = Utc::now();
    match case.status {
        CaseStatus::Noticed => {
            if let Some(deadline) = case.notice_deadline {
                if now > deadline && case.appeal_id.is_none() {
                    case.status = CaseStatus::ExpiredNoAppeal;
                }
            }
        }
        CaseStatus::AppealFiled => {
            // hearing must be scheduled within 14 days of filing
        }
        _ => {}
    }
}
