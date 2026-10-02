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

//! Integration tests for eel-abis-fp-vendor-peak.
//! 20+ tests covering policy activation, consensus paths, degrade, calibration.

use chrono::{TimeZone, Utc};
use eel_abis_fp_vendor_peak::{
    calibration::{PlattCalibrator, VendorCalibrators},
    degrade::{DegradeOutcome, DegradedCandidate},
    peak_policy::PeakPolicy,
    vendor_router::{PeakVendorRouter, VendorMatcher, VendorTriple},
    PeakError,
};

// ---------------------------------------------------------------------------
// Fixture vendors
// ---------------------------------------------------------------------------

struct ConstMatcher {
    name: String,
    healthy: bool,
    score: f64,
}

impl VendorMatcher for ConstMatcher {
    fn name(&self) -> &str {
        &self.name
    }
    fn is_healthy(&self) -> bool {
        self.healthy
    }
    fn score(&self, _probe: &[u8], _gallery: &[u8]) -> Result<f64, PeakError> {
        Ok(self.score)
    }
}

fn ts(y: i32, m: u32, d: u32) -> chrono::DateTime<chrono::Utc> {
    Utc.with_ymd_and_hms(y, m, d, 0, 0, 0).unwrap()
}

fn make_router(
    policy: PeakPolicy,
    internal_score: f64,
    a_score: f64,
    b_score: f64,
    a_healthy: bool,
    b_healthy: bool,
) -> PeakVendorRouter {
    PeakVendorRouter::new(
        policy,
        VendorCalibrators::default(),
        VendorTriple {
            internal: Box::new(ConstMatcher {
                name: "internal".into(),
                healthy: true,
                score: internal_score,
            }),
            vendor_a: Box::new(ConstMatcher {
                name: "vendor_a".into(),
                healthy: a_healthy,
                score: a_score,
            }),
            vendor_b: Box::new(ConstMatcher {
                name: "vendor_b".into(),
                healthy: b_healthy,
                score: b_score,
            }),
        },
    )
}

// ---------------------------------------------------------------------------
// PeakPolicy tests
// ---------------------------------------------------------------------------

#[test]
fn policy_freeze_active_inside_window() {
    let p = PeakPolicy::EnabledDuringFreeze {
        from: ts(2027, 8, 8),
        to: ts(2027, 8, 16),
    };
    assert!(p.is_active(ts(2027, 8, 10)));
}

#[test]
fn policy_freeze_inactive_before_window() {
    let p = PeakPolicy::EnabledDuringFreeze {
        from: ts(2027, 8, 8),
        to: ts(2027, 8, 16),
    };
    assert!(!p.is_active(ts(2027, 8, 7)));
}

#[test]
fn policy_freeze_inactive_after_window() {
    let p = PeakPolicy::EnabledDuringFreeze {
        from: ts(2027, 8, 8),
        to: ts(2027, 8, 16),
    };
    assert!(!p.is_active(ts(2027, 8, 17)));
}

#[test]
fn policy_always_enabled() {
    assert!(PeakPolicy::AlwaysEnabled.is_active(ts(2020, 1, 1)));
}

#[test]
fn policy_disabled_never_active() {
    assert!(!PeakPolicy::Disabled.is_active(ts(2027, 8, 10)));
}

#[test]
fn policy_boundary_inclusive_from() {
    let p = PeakPolicy::EnabledDuringFreeze {
        from: ts(2027, 8, 8),
        to: ts(2027, 8, 16),
    };
    assert!(p.is_active(ts(2027, 8, 8)));
}

#[test]
fn policy_boundary_inclusive_to() {
    let p = PeakPolicy::EnabledDuringFreeze {
        from: ts(2027, 8, 8),
        to: ts(2027, 8, 16),
    };
    assert!(p.is_active(ts(2027, 8, 16)));
}

// ---------------------------------------------------------------------------
// Three-vendor consensus tests
// ---------------------------------------------------------------------------

#[test]
fn three_vendor_all_agree_accepted() {
    let router = make_router(PeakPolicy::AlwaysEnabled, 0.9, 0.85, 0.88, true, true);
    let result = router.three_way_match(b"probe", b"gallery").unwrap();
    assert!(result.accepted);
    assert_eq!(result.agree_votes, 3);
    assert_eq!(result.total_vendors, 3);
}

#[test]
fn three_vendor_two_of_three_with_one_disagree_accepted() {
    // Internal and vendor_a agree (score > 0.5 after calibration), vendor_b disagrees.
    let router = make_router(PeakPolicy::AlwaysEnabled, 0.9, 0.85, 0.1, true, true);
    let result = router.three_way_match(b"probe", b"gallery").unwrap();
    assert!(result.accepted);
    assert_eq!(result.agree_votes, 2);
}

#[test]
fn three_vendor_one_of_three_rejects() {
    // Only internal agrees; vendor_a and vendor_b both score low.
    let router = make_router(PeakPolicy::AlwaysEnabled, 0.95, 0.05, 0.1, true, true);
    let err = router.three_way_match(b"probe", b"gallery").unwrap_err();
    assert!(matches!(
        err,
        PeakError::ConsensusFailed { votes: 1, total: 3 }
    ));
}

#[test]
fn three_vendor_all_disagree_rejects() {
    let router = make_router(PeakPolicy::AlwaysEnabled, 0.1, 0.05, 0.15, true, true);
    let err = router.three_way_match(b"probe", b"gallery").unwrap_err();
    assert!(matches!(err, PeakError::ConsensusFailed { .. }));
}

// ---------------------------------------------------------------------------
// Degrade path tests
// ---------------------------------------------------------------------------

#[test]
fn degrade_with_one_vendor_down_returns_two_of_two() {
    let router = make_router(PeakPolicy::AlwaysEnabled, 0.9, 0.85, 0.1, true, false);
    let result = router
        .degraded_match(b"probe", b"gallery", "vendor_b")
        .unwrap();
    assert_eq!(result.outcome, DegradeOutcome::TwoOfTwoAgreed);
    assert!(!result.is_full_confidence());
}

#[test]
fn degrade_with_two_vendors_down_single_vendor_outcome() {
    // vendor_a is unhealthy, vendor_b is explicitly unavailable.
    let router = make_router(PeakPolicy::AlwaysEnabled, 0.9, 0.1, 0.1, false, false);
    let result = router
        .degraded_match(b"probe", b"gallery", "vendor_b")
        .unwrap();
    assert_eq!(result.outcome, DegradeOutcome::SingleVendorOnly);
    assert!(!result.accepted);
}

#[test]
fn degrade_two_of_two_not_full_confidence() {
    let c = DegradedCandidate::two_of_two(0.9, true, "vendor_b".into());
    assert!(!c.is_full_confidence());
}

#[test]
fn insufficient_healthy_vendors_returns_error() {
    // All three healthy=false except internal which isn't a vendor but is in the triple.
    let router = make_router(PeakPolicy::AlwaysEnabled, 0.9, 0.5, 0.5, false, false);
    // With vendor_a and vendor_b both unhealthy, only internal is available.
    let err = router.three_way_match(b"probe", b"gallery").unwrap_err();
    assert!(matches!(err, PeakError::InsufficientVendors { .. }));
}

// ---------------------------------------------------------------------------
// Calibration tests
// ---------------------------------------------------------------------------

#[test]
fn calibration_identity_preserves_ordering() {
    let c = PlattCalibrator::identity();
    let s1 = c.calibrate(0.3).unwrap();
    let s2 = c.calibrate(0.7).unwrap();
    assert!(s2 > s1, "calibration must be monotone");
}

#[test]
fn calibration_out_of_range_low_errors() {
    let c = PlattCalibrator::identity();
    assert!(c.calibrate(-0.1).is_err());
}

#[test]
fn calibration_out_of_range_high_errors() {
    let c = PlattCalibrator::identity();
    assert!(c.calibrate(1.1).is_err());
}

#[test]
fn calibration_monotone_with_positive_slope() {
    let c = PlattCalibrator::new(8.0, -4.0);
    let scores: Vec<f64> = (0..=10).map(|i| i as f64 / 10.0).collect();
    let cal: Vec<f64> = scores.iter().map(|&s| c.calibrate(s).unwrap()).collect();
    for w in cal.windows(2) {
        assert!(w[1] >= w[0]);
    }
}

#[test]
fn vendor_calibrators_all_return_valid_range() {
    let vc = VendorCalibrators::default();
    let (ci, ca, cb) = vc.calibrate_all(0.5, 0.6, 0.7).unwrap();
    for v in [ci, ca, cb] {
        assert!((0.0..=1.0).contains(&v));
    }
}

// ---------------------------------------------------------------------------
// Policy-inactive single-vendor path
// ---------------------------------------------------------------------------

#[test]
fn disabled_policy_uses_single_vendor() {
    let router = make_router(PeakPolicy::Disabled, 0.9, 0.1, 0.1, true, true);
    let result = router.match_fingerprints(b"probe", b"gallery").unwrap();
    assert_eq!(result.total_vendors, 1);
}

#[test]
fn always_enabled_uses_three_vendors() {
    let router = make_router(PeakPolicy::AlwaysEnabled, 0.9, 0.85, 0.88, true, true);
    let result = router.match_fingerprints(b"probe", b"gallery").unwrap();
    assert_eq!(result.total_vendors, 3);
}
