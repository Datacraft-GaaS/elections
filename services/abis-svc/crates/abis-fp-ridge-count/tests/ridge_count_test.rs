//! Ridge counting and K-plet matching tests.

use abis_matcher::modality::fingerprint::{
    FingerPosition, FingerprintTemplate, Minutia, MinutiaeType,
};
use eel_abis_fp_ridge_count::{
    count::{count_ridges, OrientationField},
    kplet::build_kplets,
    matcher::{KPletMatcher, RidgeCountMatcher},
};

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

/// Flat orientation field: all pixels at `angle` degrees.
fn flat_field(width: usize, height: usize, angle: f32) -> OrientationField {
    OrientationField::new(width, height, vec![angle; width * height])
}

/// Synthetic template: N minutiae spread along x-axis at y=250 with theta=90.
fn linear_template(n: usize) -> FingerprintTemplate {
    let minutiae = (0..n)
        .map(|i| make_minutia(50 + i as u16 * 30, 250, 90.0))
        .collect();
    FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, minutiae)
}

/// Synthetic template: 10 minutiae in a 3x3 grid.
fn grid_template() -> FingerprintTemplate {
    let minutiae = (0..10)
        .map(|i| {
            let x = 100 + (i % 3) * 80;
            let y = 100 + (i / 3) * 80;
            make_minutia(x as u16, y as u16, (i * 36) as f32)
        })
        .collect();
    FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, minutiae)
}

// ---------------------------------------------------------------------------
// Ridge count tests
// ---------------------------------------------------------------------------

#[test]
fn test_count_ridges_zero_for_same_minutia() {
    let m = make_minutia(100, 100, 0.0);
    let field = flat_field(500, 500, 0.0);
    let count = count_ridges(&m, &m, &field);
    assert_eq!(count, 0);
}

#[test]
fn test_count_ridges_empty_field_uses_geometric_fallback() {
    let from = make_minutia(0, 0, 0.0);
    let to = make_minutia(150, 0, 0.0);
    let empty = OrientationField::new(0, 0, vec![]);
    let count = count_ridges(&from, &to, &empty);
    // Geometric fallback: 150 / 15 = 10
    assert_eq!(count, 10);
}

#[test]
fn test_count_ridges_monotonic_with_distance() {
    let field = flat_field(500, 500, 45.0);
    let from = make_minutia(100, 250, 0.0);

    let c1 = count_ridges(&from, &make_minutia(130, 250, 0.0), &field);
    let c2 = count_ridges(&from, &make_minutia(200, 250, 0.0), &field);
    let c3 = count_ridges(&from, &make_minutia(300, 250, 0.0), &field);
    assert!(
        c1 <= c2,
        "count should be non-decreasing with distance: {c1} > {c2}"
    );
    assert!(
        c2 <= c3,
        "count should be non-decreasing with distance: {c2} > {c3}"
    );
}

#[test]
fn test_count_ridges_positive_for_distant_minutiae() {
    let field = flat_field(500, 500, 90.0);
    let from = make_minutia(50, 250, 0.0);
    let to = make_minutia(450, 250, 0.0);
    let count = count_ridges(&from, &to, &field);
    assert!(
        count > 0,
        "ridges should be > 0 for distant minutiae, got {count}"
    );
}

#[test]
fn test_count_ridges_returns_u8() {
    let field = flat_field(500, 500, 0.0);
    let from = make_minutia(0, 0, 0.0);
    let to = make_minutia(499, 0, 0.0);
    let count = count_ridges(&from, &to, &field);
    // Just verify no panic and it's a valid u8.
    let _ = count;
}

#[test]
fn test_orientation_field_sample_clamps_bounds() {
    let field = flat_field(10, 10, 45.0);
    // Sampling outside bounds must not panic and return the boundary value.
    let v = field.sample(100.0, 100.0);
    assert!((v - 45.0).abs() < 1e-5);
}

#[test]
fn test_orientation_field_bilinear_consistent() {
    let field = flat_field(20, 20, 90.0);
    let v = field.sample(5.5, 7.5);
    assert!(
        (v - 90.0).abs() < 1e-5,
        "uniform field bilinear should give same value: {v}"
    );
}

// ---------------------------------------------------------------------------
// K-plet tests
// ---------------------------------------------------------------------------

#[test]
fn test_kplets_count_equals_minutiae_count() {
    let t = grid_template();
    let field = flat_field(500, 500, 0.0);
    let kplets = build_kplets(&t, &field);
    assert_eq!(kplets.len(), t.minutiae.len());
}

#[test]
fn test_kplet_tuples_sorted_by_distance() {
    let t = linear_template(10);
    let field = flat_field(500, 500, 0.0);
    let kplets = build_kplets(&t, &field);
    for kp in &kplets {
        let rhos: Vec<f32> = kp.tuples.iter().map(|t| t.rho).collect();
        for pair in rhos.windows(2) {
            assert!(
                pair[0] <= pair[1] + 1e-5,
                "tuples must be sorted by distance: {:?}",
                rhos
            );
        }
    }
}

#[test]
fn test_kplet_ridge_counts_nonnegative() {
    let t = linear_template(6);
    let field = flat_field(500, 500, 45.0);
    let kplets = build_kplets(&t, &field);
    for kp in &kplets {
        for tuple in &kp.tuples {
            // u8 is always >= 0; this just checks no panic.
            let _ = tuple.ridge_count;
        }
    }
}

#[test]
fn test_kplet_matcher_identical_templates_score_one() {
    let t = grid_template();
    let field = flat_field(500, 500, 0.0);
    let matcher = KPletMatcher::new(field);
    let score = matcher.compare(&t, &t);
    assert!(
        score > 0.95,
        "identical templates should score near 1.0, got {score}"
    );
}

#[test]
fn test_kplet_matcher_score_in_range() {
    let t1 = grid_template();
    let t2 = linear_template(10);
    let field = flat_field(500, 500, 0.0);
    let matcher = KPletMatcher::new(field);
    let score = matcher.compare(&t1, &t2);
    assert!((0.0..=1.0).contains(&score), "score out of range: {score}");
}

#[test]
fn test_kplet_matcher_empty_template_scores_zero() {
    let empty = FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, vec![]);
    let t = grid_template();
    let field = flat_field(500, 500, 0.0);
    let matcher = KPletMatcher::new(field);
    assert_eq!(matcher.compare(&empty, &t), 0.0);
    assert_eq!(matcher.compare(&t, &empty), 0.0);
}

#[test]
fn test_kplet_distinguishes_different_topologies() {
    // Two templates with same positions but different orientations (different ridge topology).
    let t1_minutiae: Vec<Minutia> = (0..8)
        .map(|i| make_minutia(100 + i * 40, 200, (i * 20) as f32))
        .collect();
    let t2_minutiae: Vec<Minutia> = (0..8)
        .map(|i| make_minutia(100 + i * 40, 200, (i * 20 + 90) as f32))
        .collect();
    let t1 = FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, t1_minutiae);
    let t2 = FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, t2_minutiae);

    let field = flat_field(500, 500, 0.0);
    let matcher = KPletMatcher::new(field);
    let score_same = matcher.compare(&t1, &t1);
    let score_diff = matcher.compare(&t1, &t2);
    assert!(
        score_same > score_diff,
        "same topology should score higher than rotated topology: {score_same} vs {score_diff}"
    );
}

// ---------------------------------------------------------------------------
// RidgeCountMatcher tests
// ---------------------------------------------------------------------------

#[test]
fn test_ridge_count_matcher_identical_templates_score_one() {
    let mut t = linear_template(5);
    // Populate ridge_counts with known values.
    let n = 5;
    t.ridge_counts = (0..(n * (n - 1) / 2)).map(|i| (i % 10) as u8).collect();
    let matcher = RidgeCountMatcher::default();
    let score = matcher.compare(&t, &t);
    assert!(
        score > 0.99,
        "identical ridge-count matrices should score 1.0, got {score}"
    );
}

#[test]
fn test_ridge_count_matcher_tolerance_respected() {
    let n = 4;
    let mut probe = linear_template(n);
    let mut gallery = linear_template(n);
    probe.ridge_counts = vec![3, 5, 7, 2, 4, 6];
    gallery.ridge_counts = vec![4, 5, 8, 2, 5, 6]; // within tol=2
    let matcher = RidgeCountMatcher::new(2);
    let score = matcher.compare(&probe, &gallery);
    assert!(
        score > 0.8,
        "ridge counts within tolerance should score high, got {score}"
    );
}

#[test]
fn test_ridge_count_matcher_empty_template_scores_zero() {
    let empty = FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, vec![]);
    let t = linear_template(5);
    let matcher = RidgeCountMatcher::default();
    assert_eq!(matcher.compare(&empty, &t), 0.0);
}
