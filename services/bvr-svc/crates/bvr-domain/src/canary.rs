//! Canary row detection (EC-SEC-16).
//!
//! A canary voter has `is_canary = true`. Any read path that returns a canary
//! voter MUST call `make_canary_event` and emit the resulting event so the
//! security team is alerted to unauthorized access.

use chrono::Utc;

use crate::events::CanaryTouchedV1;
use crate::voter::Voter;

/// Returns true if the voter record is a canary sentinel row.
pub fn is_canary(voter: &Voter) -> bool {
    voter.is_canary
}

/// Build a `CanaryTouchedV1` event for the given voter access.
pub fn make_canary_event(voter_id: &str, accessed_by: &str, path: &str) -> CanaryTouchedV1 {
    CanaryTouchedV1 {
        voter_id: voter_id.to_string(),
        accessed_by: accessed_by.to_string(),
        path: path.to_string(),
        ts: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voter::Voter;

    fn make_voter(canary: bool) -> Voter {
        let mut v = Voter::new(
            "KE00000001".into(),
            "Canary Voter".into(),
            "1980-01-01".into(),
            "000".into(),
            "000-001-001".into(),
            "2027-general".into(),
        );
        v.is_canary = canary;
        v
    }

    #[test]
    fn is_canary_returns_false_for_regular_voter() {
        let v = make_voter(false);
        assert!(!is_canary(&v));
    }

    #[test]
    fn is_canary_returns_true_for_canary_voter() {
        let v = make_voter(true);
        assert!(is_canary(&v));
    }

    #[test]
    fn make_canary_event_populates_fields() {
        let ev = make_canary_event("voter-123", "analyst-007", "/api/v1/voters/voter-123");
        assert_eq!(ev.voter_id, "voter-123");
        assert_eq!(ev.accessed_by, "analyst-007");
        assert_eq!(ev.path, "/api/v1/voters/voter-123");
        assert_eq!(
            crate::events::CanaryTouchedV1::event_type(),
            "eel.bvr.canary_touched.v1"
        );
    }
}
