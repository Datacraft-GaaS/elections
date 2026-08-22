//! Reconciliation arithmetic for the ballot-serial chain (WP-3.3, M63).
//!
//! FR-M63-023: unused = (serial_high - serial_low + 1) - deposited - spoilt
//! FR-M63-024: any arithmetic mismatch > 0 raises an explicit hold.
//! FR-M63-032: boundary conditions: zero spoilt, all spoilt, all unused.

use serde::{Deserialize, Serialize};

/// Input from the polling station at close of day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationCounts {
    pub station_code: String,
    /// First serial number assigned.
    pub serial_low: u64,
    /// Last serial number assigned (inclusive).
    pub serial_high: u64,
    /// Ballots deposited in the ballot box (authenticated voters).
    pub deposited_count: u64,
    /// Individually recorded spoilt ballots.
    pub spoilt_count: u64,
}

/// Result of the reconciliation arithmetic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReconciliationResult {
    pub station_code: String,
    pub assigned_count: u64,
    pub deposited_count: u64,
    pub spoilt_count: u64,
    pub unused_count: u64,
    pub discrepancy: i64,
    pub status: ReconciliationStatus,
    pub flag_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReconciliationStatus {
    Cleared,
    /// Discrepancy found; result hold raised; never silent (FR-M63-024).
    Held {
        mismatch: i64,
    },
}

/// Reconcile a station's ballot counts.  Never silent on mismatch (FR-M63-024).
pub fn reconcile(counts: &StationCounts) -> ReconciliationResult {
    // Overflow guard: serial_high must be >= serial_low.
    let assigned_count = if counts.serial_high >= counts.serial_low {
        counts.serial_high - counts.serial_low + 1
    } else {
        0
    };

    let accounted = counts.deposited_count + counts.spoilt_count;
    let unused_count = assigned_count.saturating_sub(accounted);

    // Discrepancy: accounted + unused should equal assigned.
    // Negative discrepancy means more accounted than assigned (impossible values).
    let _discrepancy = assigned_count as i64
        - (counts.deposited_count as i64 + counts.spoilt_count as i64 + unused_count as i64);

    // Recompute more carefully:
    // unused = assigned - deposited - spoilt
    let (unused_count, discrepancy, status, flag_reason) = if accounted > assigned_count {
        // Impossible: more ballots accounted than assigned; immediate hold.
        let mismatch = accounted as i64 - assigned_count as i64;
        (
            0u64,
            mismatch,
            ReconciliationStatus::Held { mismatch },
            Some(format!(
                "accounted ({accounted}) exceeds assigned ({assigned_count}); EC-DAT-11"
            )),
        )
    } else {
        let unused = assigned_count - accounted;
        // Arithmetic: deposited + spoilt + unused == assigned always true here.
        (unused, 0i64, ReconciliationStatus::Cleared, None)
    };

    ReconciliationResult {
        station_code: counts.station_code.clone(),
        assigned_count,
        deposited_count: counts.deposited_count,
        spoilt_count: counts.spoilt_count,
        unused_count,
        discrepancy,
        status,
        flag_reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_trace_rs::trace;

    fn counts(low: u64, high: u64, deposited: u64, spoilt: u64) -> StationCounts {
        StationCounts {
            station_code: "STN001".into(),
            serial_low: low,
            serial_high: high,
            deposited_count: deposited,
            spoilt_count: spoilt,
        }
    }

    #[trace("FR-M63-032")]
    #[test]
    fn zero_spoilt_clears() {
        // All ballots cast, none spoilt.
        let c = counts(1, 100, 100, 0);
        let r = reconcile(&c);
        assert_eq!(r.status, ReconciliationStatus::Cleared);
        assert_eq!(r.unused_count, 0);
        assert_eq!(r.spoilt_count, 0);
    }

    #[trace("FR-M63-032")]
    #[test]
    fn all_spoilt_zero_deposited() {
        let c = counts(1, 50, 0, 50);
        let r = reconcile(&c);
        assert_eq!(r.status, ReconciliationStatus::Cleared);
        assert_eq!(r.unused_count, 0);
    }

    #[trace("FR-M63-032")]
    #[test]
    fn all_unused() {
        let c = counts(1, 100, 0, 0);
        let r = reconcile(&c);
        assert_eq!(r.status, ReconciliationStatus::Cleared);
        assert_eq!(r.unused_count, 100);
    }

    #[trace("FR-M63-024", "EC-DAT-11")]
    #[test]
    fn over_assigned_triggers_hold() {
        // 101 deposited from a 100-ballot assignment.
        let c = counts(1, 100, 101, 0);
        let r = reconcile(&c);
        assert!(
            matches!(r.status, ReconciliationStatus::Held { mismatch: 1 }),
            "expected hold: {:?}",
            r.status
        );
    }

    #[trace("FR-M63-023")]
    #[test]
    fn partial_discrepancy_held() {
        // 100 assigned; 60 deposited + 30 spoilt = 90 accounted; 10 unused; balanced.
        let c = counts(1, 100, 60, 30);
        let r = reconcile(&c);
        assert_eq!(r.status, ReconciliationStatus::Cleared);
        assert_eq!(r.unused_count, 10);
    }

    #[trace("FR-M63-024", "EC-DAT-11")]
    #[test]
    fn both_exceed_assigned_held() {
        let c = counts(1, 100, 80, 30); // 110 > 100
        let r = reconcile(&c);
        assert!(matches!(r.status, ReconciliationStatus::Held { .. }));
        assert!(r.flag_reason.is_some());
    }
}
