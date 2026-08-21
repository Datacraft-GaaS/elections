#[tokio::test]
async fn router_smoke() {
    let app = eel_governance_referendum::router::build_router();
    drop(app);
}

// County threshold invariants -- AC per FR-M49-028, EC-LEG-07
#[test]
fn county_threshold_invariants() {
    use eel_governance_referendum::threshold::*;
    // ExactlyTwentyFourCounties: boundary
    assert!(!meets_county_threshold(23));
    assert!(meets_county_threshold(24));
    assert!(meets_county_threshold(47));
    // NoDeclarationWithoutBothThresholds
    assert!(!can_declare(23, 51, 49)); // county short
    assert!(!can_declare(24, 50, 50)); // national short
    assert!(can_declare(24, 51, 49)); // both met
    // ExactMajority: 50%+1 strictly
    assert!(!meets_national_majority(50, 50));
    assert!(meets_national_majority(51, 49));
}

// Monotonicity property: once county threshold met, stays met
#[test]
fn county_threshold_monotone() {
    use eel_governance_referendum::threshold::meets_county_threshold;
    let mut met = false;
    for c in 0u8..=47 {
        let cur = meets_county_threshold(c);
        if met { assert!(cur, "non-monotone at {c}"); }
        if cur && !met { met = true; }
    }
    assert!(met);
}

// Proptest-style exhaustive county sweep
#[test]
fn all_47_county_combos_satisfy_invariant() {
    use eel_governance_referendum::threshold::{county_count_valid, meets_county_threshold};
    for c in 0u8..=47 {
        assert!(county_count_valid(c));
        let expected = c >= 24;
        assert_eq!(meets_county_threshold(c), expected, "failed at county count {c}");
    }
}
