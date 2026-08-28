//! Time-bound identification intake guard (HM2 geofence/time).
//!
//! `EEL_POLL_WINDOW` (RFC3339,RFC3339) defines the open/close instants during
//! which identification intake is unconditionally accepted. Outside the
//! window the guard distinguishes boundary clock drift (within a skew
//! tolerance, default 120s from `EEL_INTAKE_SKEW_SECS`) from genuine
//! out-of-hours submissions:
//!
//! - inside window            -> `Decision::Open` (proceed unchanged)
//! - outside, within skew     -> `Decision::Review { drift_secs }`
//!                               (proceed but flagged for human review)
//! - outside, beyond hard limit -> reject (`BeforePoll` / `AfterPoll`)
//! - window unset             -> `Decision::Open` (behaviour identical to
//!                               today; existing deployments unaffected)

use std::sync::OnceLock;

use chrono::{DateTime, Utc};

/// Default clock-skew tolerance at either edge of the poll window.
pub const DEFAULT_SKEW_SECS: i64 = 120;

/// Verdict for one intake attempt against the configured poll window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Inside the window (or no window configured): proceed normally.
    Open,
    /// Before the window opened by more than the hard limit: reject.
    BeforePoll { opens_in_secs: i64 },
    /// After the window closed by more than the hard limit: reject.
    AfterPoll { closed_ago_secs: i64 },
    /// Just outside the window within skew tolerance: proceed for review.
    Review { drift_secs: i64 },
}

static WINDOW: OnceLock<Option<(DateTime<Utc>, DateTime<Utc>)>> = OnceLock::new();
static SKEW: OnceLock<i64> = OnceLock::new();

/// Poll window parsed once from `EEL_POLL_WINDOW`; `None` when unset/invalid.
pub fn poll_window() -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    *WINDOW.get_or_init(|| {
        std::env::var("EEL_POLL_WINDOW")
            .ok()
            .and_then(|raw| parse_window(&raw))
    })
}

/// Skew tolerance parsed once from `EEL_INTAKE_SKEW_SECS`.
pub fn skew_secs() -> i64 {
    *SKEW.get_or_init(|| {
        std::env::var("EEL_INTAKE_SKEW_SECS")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(DEFAULT_SKEW_SECS)
    })
}

/// Parse `"RFC3339,RFC3339"` into `(open, close)`; rejects inverted windows.
pub fn parse_window(raw: &str) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let (a, b) = raw.split_once(',')?;
    let open = DateTime::parse_from_rfc3339(a.trim()).ok()?.with_timezone(&Utc);
    let close = DateTime::parse_from_rfc3339(b.trim()).ok()?.with_timezone(&Utc);
    (open <= close).then_some((open, close))
}

/// Evaluate with the default skew tolerance.
pub fn check_window(
    window: Option<(DateTime<Utc>, DateTime<Utc>)>,
    now: DateTime<Utc>,
) -> Decision {
    check_window_skewed(window, now, DEFAULT_SKEW_SECS)
}

/// Evaluate with an explicit skew tolerance. Drift beyond `skew_secs` is a
/// hard reject (which subsumes the documented `2 * skew` hard limit).
pub fn check_window_skewed(
    window: Option<(DateTime<Utc>, DateTime<Utc>)>,
    now: DateTime<Utc>,
    skew_secs: i64,
) -> Decision {
    let Some((open, close)) = window else {
        return Decision::Open;
    };
    if now >= open && now <= close {
        return Decision::Open;
    }
    if now < open {
        let opens_in_secs = (open - now).num_seconds();
        if opens_in_secs <= skew_secs {
            return Decision::Review {
                drift_secs: opens_in_secs,
            };
        }
        return Decision::BeforePoll { opens_in_secs };
    }
    let closed_ago_secs = (now - close).num_seconds();
    if closed_ago_secs <= skew_secs {
        return Decision::Review {
            drift_secs: closed_ago_secs,
        };
    }
    Decision::AfterPoll { closed_ago_secs }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn base() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 1, 8, 0, 0).unwrap()
    }

    fn window() -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        Some((base(), base() + chrono::Duration::hours(10)))
    }

    fn at(secs: i64) -> DateTime<Utc> {
        base() + chrono::Duration::seconds(secs)
    }

    #[test]
    fn inside_window_is_open() {
        assert_eq!(check_window(window(), at(0)), Decision::Open);
        assert_eq!(check_window(window(), at(36_000)), Decision::Open);
        assert_eq!(check_window(window(), at(18_000)), Decision::Open);
    }

    #[test]
    fn unset_window_is_always_open() {
        assert_eq!(check_window(None, at(-86_400)), Decision::Open);
        assert_eq!(check_window(None, at(86_400)), Decision::Open);
    }

    #[test]
    fn skew_inside_is_review_with_drift() {
        // 90s before open: within the 120s default skew -> review, not reject.
        assert_eq!(
            check_window(window(), at(-90)),
            Decision::Review { drift_secs: 90 }
        );
        // Boundary equality counts as review.
        assert_eq!(
            check_window_skewed(window(), at(-120), 120),
            Decision::Review { drift_secs: 120 }
        );
        // 45s after close.
        assert_eq!(
            check_window(window(), at(36_045)),
            Decision::Review { drift_secs: 45 }
        );
    }

    #[test]
    fn beyond_two_skew_is_rejected() {
        // Far before open (> 2 * skew): hard reject with time-to-open.
        match check_window(window(), at(-600)) {
            Decision::BeforePoll { opens_in_secs } => assert_eq!(opens_in_secs, 600),
            other => panic!("expected BeforePoll, got {other:?}"),
        }
        // Far after close (> 2 * skew): hard reject with time-since-close.
        match check_window(window(), at(40_000)) {
            Decision::AfterPoll { closed_ago_secs } => assert_eq!(closed_ago_secs, 4_000),
            other => panic!("expected AfterPoll, got {other:?}"),
        }
        // Between skew and 2*skew is also rejected under this policy.
        assert!(matches!(
            check_window_skewed(window(), at(-200), 100),
            Decision::BeforePoll { .. }
        ));
    }

    #[test]
    fn custom_skew_tolerance_is_honoured() {
        assert_eq!(
            check_window_skewed(window(), at(-300), 300),
            Decision::Review { drift_secs: 300 }
        );
        assert!(matches!(
            check_window_skewed(window(), at(-301), 300),
            Decision::BeforePoll { .. }
        ));
    }

    #[test]
    fn parse_window_roundtrip_and_rejects_inverted() {
        let raw = "2026-03-01T08:00:00Z,2026-03-01T18:00:00Z";
        let w = parse_window(raw).expect("must parse");
        assert_eq!(w.0, base());
        assert_eq!(w.1, base() + chrono::Duration::hours(10));
        assert!(parse_window("2026-03-01T18:00:00Z,2026-03-01T08:00:00Z").is_none());
        assert!(parse_window("not-a-window").is_none());
        assert!(parse_window("").is_none());
    }
}
