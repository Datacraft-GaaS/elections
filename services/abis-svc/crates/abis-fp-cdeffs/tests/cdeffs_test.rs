//! CDEFFS integration tests.

use eel_abis_fp_cdeffs::{
    features::{
        Dot, ExtendedFeatureSet, IncipientRidge, MinutiaExtended, PatternClass, RidgeEdgeShape,
        RidgeFlowMap, RidgeQualityMap, SingularPoint, SingularPointKind, SkinCondition,
        SkinConditionKind,
    },
    parse_efs, serialise_efs, CdeffsMatcher,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn golden_efs() -> ExtendedFeatureSet {
    let mut efs = ExtendedFeatureSet::new(PatternClass::LeftLoop);
    efs.singular_points.push(SingularPoint {
        kind: SingularPointKind::Core,
        x: 250,
        y: 200,
        orientation: 90.0,
    });
    efs.singular_points.push(SingularPoint {
        kind: SingularPointKind::Delta,
        x: 150,
        y: 350,
        orientation: 270.0,
    });
    efs.quality_map = Some(RidgeQualityMap {
        width: 2,
        height: 2,
        data: vec![80, 70, 60, 90],
    });
    efs.flow_map = Some(RidgeFlowMap {
        width: 2,
        height: 2,
        data: vec![45, 90, 135, 0],
    });
    efs.minutia_extended.push(MinutiaExtended {
        minutia_idx: 0,
        ridge_counts_to_neighbours: vec![(1, 3), (2, 5)],
        feature_type: 1,
        is_dot: false,
        is_incipient: false,
    });
    efs.dots.push(Dot {
        x: 100,
        y: 100,
        quality: 60,
    });
    efs.incipient_ridges.push(IncipientRidge {
        start_x: 10,
        start_y: 20,
        end_x: 50,
        end_y: 60,
        quality: 70,
    });
    efs.skin_conditions.push(SkinCondition {
        kind: SkinConditionKind::Crease,
        x: 30,
        y: 40,
        width: 10,
        height: 5,
    });
    efs.ridge_edge_shapes.push(RidgeEdgeShape {
        minutia_idx: 0,
        curvature: 1.5,
        ridge_width: 8.2,
    });
    efs
}

// ---------------------------------------------------------------------------
// Parse/serialise round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_serialise_parse_round_trip_byte_identical() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let bytes2 = serialise_efs(&parse_efs(&bytes).unwrap());
    assert_eq!(bytes, bytes2, "round-trip must be byte-identical");
}

#[test]
fn test_parse_pattern_class() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    assert_eq!(parsed.pattern_class, PatternClass::LeftLoop);
}

#[test]
fn test_parse_singular_point_count() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    assert_eq!(parsed.singular_points.len(), 2);
}

#[test]
fn test_parse_core_kind() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    assert_eq!(parsed.singular_points[0].kind, SingularPointKind::Core);
}

#[test]
fn test_parse_delta_kind() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    assert_eq!(parsed.singular_points[1].kind, SingularPointKind::Delta);
}

#[test]
fn test_parse_singular_point_position() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    assert_eq!(parsed.singular_points[0].x, 250);
    assert_eq!(parsed.singular_points[0].y, 200);
}

#[test]
fn test_parse_singular_point_orientation() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    let orient = parsed.singular_points[0].orientation;
    assert!(
        (orient - 90.0).abs() < 0.1,
        "orientation round-trip: {orient}"
    );
}

#[test]
fn test_parse_quality_map() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    let qm = parsed.quality_map.unwrap();
    assert_eq!(qm.width, 2);
    assert_eq!(qm.height, 2);
    assert_eq!(qm.data, vec![80, 70, 60, 90]);
}

#[test]
fn test_parse_flow_map() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    let fm = parsed.flow_map.unwrap();
    assert_eq!(fm.data, vec![45, 90, 135, 0]);
}

#[test]
fn test_parse_minutia_extended_count() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    assert_eq!(parsed.minutia_extended.len(), 1);
}

#[test]
fn test_parse_minutia_extended_ridge_counts() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    let me = &parsed.minutia_extended[0];
    assert_eq!(me.ridge_counts_to_neighbours, vec![(1, 3), (2, 5)]);
}

#[test]
fn test_parse_dot() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    assert_eq!(parsed.dots.len(), 1);
    assert_eq!(parsed.dots[0].x, 100);
    assert_eq!(parsed.dots[0].quality, 60);
}

#[test]
fn test_parse_incipient_ridge() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    assert_eq!(parsed.incipient_ridges.len(), 1);
    assert_eq!(parsed.incipient_ridges[0].start_x, 10);
    assert_eq!(parsed.incipient_ridges[0].end_y, 60);
}

#[test]
fn test_parse_skin_condition() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    assert_eq!(parsed.skin_conditions.len(), 1);
}

#[test]
fn test_parse_ridge_edge_shape() {
    let efs = golden_efs();
    let bytes = serialise_efs(&efs);
    let parsed = parse_efs(&bytes).unwrap();
    assert_eq!(parsed.ridge_edge_shapes.len(), 1);
    let re = &parsed.ridge_edge_shapes[0];
    assert!((re.curvature - 1.5).abs() < 1e-5);
    assert!((re.ridge_width - 8.2).abs() < 1e-4);
}

// ---------------------------------------------------------------------------
// Matcher tests
// ---------------------------------------------------------------------------

#[test]
fn test_matcher_same_class_no_penalty() {
    let efs = golden_efs();
    let m = CdeffsMatcher::new();
    let score = m.fuse(0.7, &efs, &efs);
    assert!(
        score >= 0.7,
        "same class should not reduce score, got {score}"
    );
}

#[test]
fn test_matcher_class_mismatch_whorl_vs_arch_returns_zero() {
    let probe = ExtendedFeatureSet::new(PatternClass::Whorl);
    let gallery = ExtendedFeatureSet::new(PatternClass::Arch);
    let m = CdeffsMatcher::new();
    let score = m.fuse(0.8, &probe, &gallery);
    assert_eq!(
        score, 0.0,
        "Whorl vs Arch must return 0.0 fusion, got {score}"
    );
}

#[test]
fn test_matcher_class_mismatch_left_loop_vs_right_loop() {
    let probe = ExtendedFeatureSet::new(PatternClass::LeftLoop);
    let gallery = ExtendedFeatureSet::new(PatternClass::RightLoop);
    let score = CdeffsMatcher::new().fuse(0.6, &probe, &gallery);
    assert_eq!(
        score, 0.0,
        "LeftLoop vs RightLoop must return 0.0, got {score}"
    );
}

#[test]
fn test_matcher_singular_point_agreement_gives_bonus() {
    let efs = golden_efs();
    let base = 0.6_f32;
    let score = CdeffsMatcher::new().fuse(base, &efs, &efs);
    assert!(
        score > base,
        "matched singular points should give bonus, got {score}"
    );
}

#[test]
fn test_matcher_score_clamped_to_one() {
    let efs = golden_efs();
    let score = CdeffsMatcher::new().fuse(1.0, &efs, &efs);
    assert!(score <= 1.0, "score must not exceed 1.0, got {score}");
}

#[test]
fn test_matcher_score_nonnegative() {
    let efs = golden_efs();
    let score = CdeffsMatcher::new().fuse(0.0, &efs, &efs);
    assert!(score >= 0.0, "score must be >= 0.0, got {score}");
}

#[test]
fn test_empty_efs_round_trip() {
    let efs = ExtendedFeatureSet::new(PatternClass::Arch);
    let bytes = serialise_efs(&efs);
    let bytes2 = serialise_efs(&parse_efs(&bytes).unwrap());
    assert_eq!(bytes, bytes2);
}

#[test]
fn test_core_count() {
    let efs = golden_efs();
    assert_eq!(efs.core_count(), 1);
}

#[test]
fn test_delta_count() {
    let efs = golden_efs();
    assert_eq!(efs.delta_count(), 1);
}

#[test]
fn test_compatible_whorl_with_twin_loop() {
    let probe = ExtendedFeatureSet::new(PatternClass::Whorl);
    let gallery = ExtendedFeatureSet::new(PatternClass::TwinLoop);
    let score = CdeffsMatcher::new().fuse(0.5, &probe, &gallery);
    assert!(
        score >= 0.5,
        "Whorl and TwinLoop are compatible, got {score}"
    );
}
