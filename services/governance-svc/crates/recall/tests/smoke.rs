#[tokio::test]
async fn router_smoke() {
    let app = eel_governance_recall::router::build_router();
    drop(app);
}

#[test]
fn threshold_property_boundaries() {
    use eel_governance_recall::threshold::*;
    // Boundary: exactly 30%
    assert!(meets_art_104_2_threshold(30, 100));
    // Below threshold
    assert!(!meets_art_104_2_threshold(29, 100));
    // Above threshold
    assert!(meets_art_104_2_threshold(31, 100));
    // Large constituency boundary
    assert!(meets_art_104_2_threshold(90_000, 300_000));
    assert!(!meets_art_104_2_threshold(89_999, 300_000));
}

#[test]
fn art_104_3_property_boundaries() {
    use eel_governance_recall::threshold::*;
    // Majority: yes > 50% of registered voters
    assert!(meets_art_104_3_threshold(51, 100));
    assert!(!meets_art_104_3_threshold(50, 100)); // not majority
    assert!(!meets_art_104_3_threshold(49, 100));
}

#[test]
fn proptest_threshold_monotone() {
    use eel_governance_recall::threshold::meets_art_104_2_threshold;
    // If meets threshold at k signatories, should also meet at k+1
    for registered in [100u64, 1_000, 10_000, 300_000] {
        let mut found_threshold = false;
        for sig in 0..=registered {
            let current = meets_art_104_2_threshold(sig, registered);
            if found_threshold {
                // Once threshold met, all higher values must also meet it
                assert!(
                    current,
                    "monotonicity violated at sig={sig}, registered={registered}"
                );
            }
            if current && !found_threshold {
                found_threshold = true;
            }
        }
    }
}
