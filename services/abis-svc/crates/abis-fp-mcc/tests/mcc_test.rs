//! MCC integration tests.

use std::f32::consts::PI;

use abis_matcher::modality::fingerprint::{
    FingerPosition, FingerprintTemplate, Minutia, MinutiaeType,
};
use eel_abis_fp_mcc::{build_mcc, MccMatcher, MccParams};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_minutia(x: u16, y: u16, theta: f32) -> Minutia {
    Minutia {
        x,
        y,
        theta,
        mtype: MinutiaeType::Ending,
        quality: 80,
    }
}

fn make_bif(x: u16, y: u16, theta: f32) -> Minutia {
    Minutia {
        x,
        y,
        theta,
        mtype: MinutiaeType::Bifurcation,
        quality: 75,
    }
}

/// Synthetic 20-minutia template spread evenly in a 400x400 region.
fn synth_template_20() -> FingerprintTemplate {
    let minutiae = (0..20)
        .map(|i| {
            let x = 50 + (i % 4) * 90;
            let y = 50 + (i / 4) * 90;
            let theta = (i as f32 * 18.0) % 360.0;
            make_minutia(x as u16, y as u16, theta)
        })
        .collect();
    FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, minutiae)
}

/// Clone the template and rotate all orientations by `delta_deg` degrees.
fn rotate_template(t: &FingerprintTemplate, delta_deg: f32) -> FingerprintTemplate {
    let rad = delta_deg * PI / 180.0;
    let cos_r = rad.cos();
    let sin_r = rad.sin();
    let cx = t.image_width as f32 / 2.0;
    let cy = t.image_height as f32 / 2.0;
    let minutiae = t
        .minutiae
        .iter()
        .map(|m| {
            let dx = m.x as f32 - cx;
            let dy = m.y as f32 - cy;
            let nx = (cx + dx * cos_r - dy * sin_r).clamp(0.0, t.image_width as f32 - 1.0) as u16;
            let ny = (cy + dx * sin_r + dy * cos_r).clamp(0.0, t.image_height as f32 - 1.0) as u16;
            let ntheta = (m.theta + delta_deg).rem_euclid(360.0);
            Minutia {
                x: nx,
                y: ny,
                theta: ntheta,
                ..*m
            }
        })
        .collect();
    FingerprintTemplate::from_minutiae(t.position, minutiae)
}

// ---------------------------------------------------------------------------
// Tests: cylinder construction
// ---------------------------------------------------------------------------

#[test]
fn test_build_produces_correct_cylinder_count() {
    let t = synth_template_20();
    let params = MccParams::default();
    let cyls = build_mcc(&t, &params);
    assert_eq!(cyls.len(), 20, "one cylinder per minutia");
}

#[test]
fn test_cylinder_bit_vector_length() {
    let t = synth_template_20();
    let params = MccParams::default();
    let cyls = build_mcc(&t, &params);
    let ns = params.ns as usize;
    let nd = params.nd as usize;
    let expected_bytes = (ns * ns * nd + 7) / 8;
    for c in &cyls {
        assert_eq!(c.cells.len(), expected_bytes);
    }
}

#[test]
fn test_cylinder_cell_count() {
    let params = MccParams::default();
    let t = synth_template_20();
    let cyls = build_mcc(&t, &params);
    let expected = params.ns as usize * params.ns as usize * params.nd as usize;
    for c in &cyls {
        assert_eq!(c.cell_count(), expected);
    }
}

#[test]
fn test_single_minutia_template_all_zero_cylinders() {
    // With only one minutia there are no neighbours; all cells must be 0.
    let t = FingerprintTemplate::from_minutiae(
        FingerPosition::RightIndex,
        vec![make_minutia(250, 250, 45.0)],
    );
    let cyls = build_mcc(&t, &MccParams::default());
    assert_eq!(cyls.len(), 1);
    assert_eq!(cyls[0].popcount, 0);
}

#[test]
fn test_popcount_consistency() {
    let t = synth_template_20();
    let cyls = build_mcc(&t, &MccParams::default());
    for c in &cyls {
        let actual: u32 = c.cells.iter().map(|b| b.count_ones()).sum();
        assert_eq!(c.popcount, actual);
    }
}

#[test]
fn test_radius_stored_correctly() {
    let mut params = MccParams::default();
    params.radius = 55.0;
    let t = synth_template_20();
    let cyls = build_mcc(&t, &params);
    for c in &cyls {
        assert!((c.radius - 55.0).abs() < 1e-5);
    }
}

#[test]
fn test_nd_stored_correctly() {
    let params = MccParams::default();
    let t = synth_template_20();
    let cyls = build_mcc(&t, &params);
    for c in &cyls {
        assert_eq!(c.nd, params.nd);
    }
}

#[test]
fn test_ns_stored_correctly() {
    let params = MccParams::default();
    let t = synth_template_20();
    let cyls = build_mcc(&t, &params);
    for c in &cyls {
        assert_eq!(c.ns, params.ns);
    }
}

// ---------------------------------------------------------------------------
// Tests: similarity
// ---------------------------------------------------------------------------

#[test]
fn test_identical_templates_score_one() {
    let t = synth_template_20();
    let matcher = MccMatcher::default();
    let score = matcher.compare(&t, &t);
    assert!(
        score > 0.99,
        "identical templates must score ~1.0, got {score}"
    );
}

#[test]
fn test_rotated_15_deg_still_high_score() {
    let t = synth_template_20();
    let t2 = rotate_template(&t, 15.0);
    let matcher = MccMatcher::default();
    let score = matcher.compare(&t, &t2);
    assert!(
        score > 0.85,
        "15-deg rotation should score >0.85, got {score}"
    );
}

#[test]
fn test_rotated_5_deg_high_score() {
    let t = synth_template_20();
    let t2 = rotate_template(&t, 5.0);
    let score = MccMatcher::default().compare(&t, &t2);
    assert!(
        score > 0.90,
        "5-deg rotation should score >0.90, got {score}"
    );
}

#[test]
fn test_missing_5_minutiae_still_acceptable() {
    let t = synth_template_20();
    let mut t2 = t.clone();
    t2.minutiae.truncate(15); // remove 5
    let score = MccMatcher::default().compare(&t, &t2);
    assert!(
        score > 0.60,
        "5 missing minutiae should score >0.60, got {score}"
    );
}

#[test]
fn test_spurious_5_minutiae_still_acceptable() {
    let t = synth_template_20();
    let mut t2 = t.clone();
    // Add 5 spurious minutiae far from real ones.
    for i in 0..5 {
        t2.minutiae
            .push(make_minutia(10 + i * 7, 10, (i * 33) as f32));
    }
    let score = MccMatcher::default().compare(&t, &t2);
    assert!(
        score > 0.60,
        "5 spurious minutiae should score >0.60, got {score}"
    );
}

#[test]
fn test_disjoint_templates_low_score() {
    // Two templates with distinct LOCAL orientation structures:
    // Template 1: radial orientations (all minutiae point away from the centre).
    // Template 2: all minutiae aligned to a single fixed orientation (90 deg).
    // Their MCC cylinders capture these different local patterns; cross-match
    // should score substantially below the self-match.
    let t1_minutiae: Vec<Minutia> = (0..20)
        .map(|i| {
            let angle = (i as f32 / 20.0) * 360.0;
            let r = 60.0_f32;
            let x = (250.0 + r * angle.to_radians().cos()) as u16;
            let y = (250.0 + r * angle.to_radians().sin()) as u16;
            make_minutia(x, y, angle % 360.0) // radial orientation
        })
        .collect();
    let t2_minutiae: Vec<Minutia> = (0..20)
        .map(|i| {
            let angle = (i as f32 / 20.0) * 360.0;
            let r = 60.0_f32;
            let x = (250.0 + r * angle.to_radians().cos()) as u16;
            let y = (250.0 + r * angle.to_radians().sin()) as u16;
            make_minutia(x, y, 90.0) // all same direction -- orthogonal to radial at every point
        })
        .collect();

    let mut params = MccParams::default();
    params.sigma_d = std::f32::consts::PI / 12.0; // tighter directional bins for more discrimination
    let matcher = MccMatcher::new(params);

    let t1 = FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, t1_minutiae);
    let t2 = FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, t2_minutiae);

    let score_same = matcher.compare(&t1, &t1);
    let score_diff = matcher.compare(&t1, &t2);

    assert!(
        score_diff < score_same * 0.6,
        "structurally different templates should score well below self-match ({score_same}), got cross={score_diff}"
    );
}

#[test]
fn test_empty_template_scores_zero() {
    let empty = FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, vec![]);
    let t = synth_template_20();
    assert_eq!(MccMatcher::default().compare(&empty, &t), 0.0);
    assert_eq!(MccMatcher::default().compare(&t, &empty), 0.0);
}

#[test]
fn test_score_symmetric() {
    let t1 = synth_template_20();
    let t2 = rotate_template(&t1, 10.0);
    let s1 = MccMatcher::default().compare(&t1, &t2);
    let s2 = MccMatcher::default().compare(&t2, &t1);
    assert!(
        (s1 - s2).abs() < 0.01,
        "score must be symmetric, got {s1} vs {s2}"
    );
}

#[test]
fn test_local_similarity_self_is_one() {
    let t = synth_template_20();
    let cyls = build_mcc(&t, &MccParams::default());
    for c in &cyls {
        if c.popcount > 0 {
            let sim = eel_abis_fp_mcc::local_similarity(c, c);
            assert!(
                (sim - 1.0).abs() < 1e-5,
                "self-similarity must be 1.0, got {sim}"
            );
        }
    }
}

#[test]
fn test_local_similarity_range() {
    let t1 = synth_template_20();
    let t2 = rotate_template(&t1, 10.0);
    let cyls1 = build_mcc(&t1, &MccParams::default());
    let cyls2 = build_mcc(&t2, &MccParams::default());
    for c1 in &cyls1 {
        for c2 in &cyls2 {
            let s = eel_abis_fp_mcc::local_similarity(c1, c2);
            assert!((0.0..=1.0).contains(&s), "similarity out of range: {s}");
        }
    }
}

#[test]
fn test_custom_params_smaller_radius() {
    let t = synth_template_20();
    let mut params = MccParams::default();
    params.radius = 40.0;
    let cyls = build_mcc(&t, &params);
    assert_eq!(cyls.len(), 20);
}

#[test]
fn test_global_similarity_identical() {
    let t = synth_template_20();
    let cyls = build_mcc(&t, &MccParams::default());
    let s = eel_abis_fp_mcc::global_similarity(&cyls, &cyls, 4);
    assert!(
        s > 0.99,
        "identical cylinder sets should score ~1.0, got {s}"
    );
}

#[test]
fn test_mixed_minutia_types() {
    let minutiae = vec![
        make_minutia(100, 100, 0.0),
        make_bif(200, 100, 90.0),
        make_minutia(150, 200, 45.0),
        make_bif(100, 200, 180.0),
        make_minutia(200, 200, 270.0),
    ];
    let t = FingerprintTemplate::from_minutiae(FingerPosition::LeftIndex, minutiae);
    let cyls = build_mcc(&t, &MccParams::default());
    assert_eq!(cyls.len(), 5);
}

#[test]
fn test_min_matched_threshold_respected() {
    // With min_matched=100 (impossible), score should be 0.
    let t = synth_template_20();
    let cyls = build_mcc(&t, &MccParams::default());
    let s = eel_abis_fp_mcc::global_similarity(&cyls, &cyls, 100);
    assert_eq!(s, 0.0, "should return 0 when fewer pairs than min_matched");
}

#[test]
fn test_score_clamps_to_one() {
    let t = synth_template_20();
    let score = MccMatcher::default().compare(&t, &t);
    assert!(score <= 1.0, "score must not exceed 1.0, got {score}");
}

#[test]
fn test_score_nonnegative() {
    let t1 = synth_template_20();
    let t2 = rotate_template(&t1, 90.0);
    let score = MccMatcher::default().compare(&t1, &t2);
    assert!(score >= 0.0, "score must not be negative, got {score}");
}

#[test]
fn test_different_finger_positions_still_compares() {
    let t1 = synth_template_20();
    let mut t2 = t1.clone();
    t2.position = FingerPosition::LeftThumb;
    // Score computation must not panic regardless of position field.
    let _ = MccMatcher::default().compare(&t1, &t2);
}
