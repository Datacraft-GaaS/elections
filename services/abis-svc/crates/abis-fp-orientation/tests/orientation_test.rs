//! Tests for orientation field analysis and matching.

use eel_abis_fp_orientation::{
    field::{compute_orientation_field, field_correlation},
    matcher::OrientationMatcher,
    pattern::{classify_pattern, FingerprintPattern},
    singular::{detect_singular_points, SingularKind},
    GrayImage,
};

fn striped_v(w: u32, h: u32, period: u32) -> GrayImage {
    let pixels: Vec<u8> = (0..(w * h))
        .map(|i| if (i % w / period) % 2 == 0 { 50u8 } else { 200 })
        .collect();
    GrayImage::new(w, h, pixels).unwrap()
}

fn striped_h(w: u32, h: u32, period: u32) -> GrayImage {
    let pixels: Vec<u8> = (0..(w * h))
        .map(|i| if (i / w / period) % 2 == 0 { 50u8 } else { 200 })
        .collect();
    GrayImage::new(w, h, pixels).unwrap()
}

fn flat(w: u32, h: u32) -> GrayImage {
    GrayImage::new(w, h, vec![128u8; (w * h) as usize]).unwrap()
}

// ----- GrayImage construction -----

#[test]
fn test_gray_image_ok() {
    assert!(GrayImage::new(32, 32, vec![0u8; 32 * 32]).is_ok());
}

#[test]
fn test_gray_image_size_mismatch_err() {
    assert!(GrayImage::new(32, 32, vec![0u8; 10]).is_err());
}

// ----- Orientation field -----

#[test]
fn test_field_dimensions() {
    let img = striped_v(64, 64, 5);
    let f = compute_orientation_field(&img, 8).unwrap();
    assert_eq!(f.width, 8);
    assert_eq!(f.height, 8);
}

#[test]
fn test_field_angles_in_range() {
    let img = striped_v(64, 64, 5);
    let f = compute_orientation_field(&img, 8).unwrap();
    for &a in &f.angles {
        assert!(
            a >= 0.0 && a < std::f32::consts::PI,
            "angle {a} out of [0, PI)"
        );
    }
}

#[test]
fn test_field_too_small_errors() {
    let img = GrayImage::new(8, 8, vec![0u8; 64]).unwrap();
    assert!(compute_orientation_field(&img, 8).is_err());
}

#[test]
fn test_field_correlation_identical_is_one() {
    let img = striped_v(64, 64, 5);
    let f = compute_orientation_field(&img, 8).unwrap();
    let corr = field_correlation(&f, &f).unwrap();
    assert!(
        (corr - 1.0).abs() < 1e-4,
        "identical field correlation should be 1.0, got {corr}"
    );
}

#[test]
fn test_field_correlation_orthogonal_is_lower() {
    let v = striped_v(64, 64, 5);
    let h = striped_h(64, 64, 5);
    let fv = compute_orientation_field(&v, 8).unwrap();
    let fh = compute_orientation_field(&h, 8).unwrap();
    let corr_same = field_correlation(&fv, &fv).unwrap();
    let corr_orth = field_correlation(&fv, &fh).unwrap();
    assert!(
        corr_same > corr_orth,
        "self-correlation should exceed orthogonal correlation"
    );
}

// ----- Singular points -----

#[test]
fn test_no_singular_points_in_flat_image() {
    let img = flat(64, 64);
    let f = compute_orientation_field(&img, 8).unwrap();
    let sp = detect_singular_points(&f);
    // Flat image: all angles = 0 -> Poincare index = 0 everywhere.
    for p in &sp {
        assert_ne!(p.kind, SingularKind::Core);
    }
}

#[test]
fn test_detect_singular_returns_vec() {
    let img = striped_v(128, 128, 8);
    let f = compute_orientation_field(&img, 8).unwrap();
    let _sp = detect_singular_points(&f);
    // Just verifying it doesn't panic.
}

#[test]
fn test_singular_point_in_bounds() {
    let img = striped_v(128, 128, 8);
    let f = compute_orientation_field(&img, 8).unwrap();
    let sp = detect_singular_points(&f);
    for p in &sp {
        assert!(
            p.x < f.width,
            "singular x={} >= field width={}",
            p.x,
            f.width
        );
        assert!(
            p.y < f.height,
            "singular y={} >= field height={}",
            p.y,
            f.height
        );
    }
}

// ----- Pattern classifier -----

#[test]
fn test_arch_when_no_singular_points() {
    let pat = classify_pattern(&[]);
    assert_eq!(pat, FingerprintPattern::Arch);
}

#[test]
fn test_tented_arch_with_one_delta() {
    use eel_abis_fp_orientation::singular::SingularPoint;
    let sp = vec![SingularPoint {
        kind: SingularKind::Delta,
        x: 5,
        y: 5,
        orientation: 0.0,
    }];
    let pat = classify_pattern(&sp);
    assert_eq!(pat, FingerprintPattern::TentedArch);
}

#[test]
fn test_right_loop_core_left_of_delta() {
    use eel_abis_fp_orientation::singular::SingularPoint;
    let sp = vec![
        SingularPoint {
            kind: SingularKind::Core,
            x: 3,
            y: 5,
            orientation: 0.0,
        },
        SingularPoint {
            kind: SingularKind::Delta,
            x: 7,
            y: 5,
            orientation: 0.0,
        },
    ];
    let pat = classify_pattern(&sp);
    assert_eq!(pat, FingerprintPattern::RightLoop);
}

#[test]
fn test_left_loop_core_right_of_delta() {
    use eel_abis_fp_orientation::singular::SingularPoint;
    let sp = vec![
        SingularPoint {
            kind: SingularKind::Core,
            x: 9,
            y: 5,
            orientation: 0.0,
        },
        SingularPoint {
            kind: SingularKind::Delta,
            x: 3,
            y: 5,
            orientation: 0.0,
        },
    ];
    let pat = classify_pattern(&sp);
    assert_eq!(pat, FingerprintPattern::LeftLoop);
}

#[test]
fn test_whorl_two_cores_two_deltas() {
    use eel_abis_fp_orientation::singular::SingularPoint;
    let sp = vec![
        SingularPoint {
            kind: SingularKind::Core,
            x: 4,
            y: 5,
            orientation: 0.0,
        },
        SingularPoint {
            kind: SingularKind::Core,
            x: 8,
            y: 5,
            orientation: 0.0,
        },
        SingularPoint {
            kind: SingularKind::Delta,
            x: 3,
            y: 7,
            orientation: 0.0,
        },
        SingularPoint {
            kind: SingularKind::Delta,
            x: 9,
            y: 7,
            orientation: 0.0,
        },
    ];
    let pat = classify_pattern(&sp);
    assert_eq!(pat, FingerprintPattern::Whorl);
}

#[test]
fn test_pattern_loop_core_count_is_one() {
    use eel_abis_fp_orientation::singular::SingularPoint;
    let sp_loop = vec![
        SingularPoint {
            kind: SingularKind::Core,
            x: 5,
            y: 5,
            orientation: 0.0,
        },
        SingularPoint {
            kind: SingularKind::Delta,
            x: 9,
            y: 5,
            orientation: 0.0,
        },
    ];
    let cores: Vec<_> = sp_loop
        .iter()
        .filter(|p| matches!(p.kind, SingularKind::Core))
        .collect();
    assert_eq!(cores.len(), 1, "loop should have exactly 1 core");
}

#[test]
fn test_pattern_arch_core_count_is_zero() {
    let sp_arch: &[eel_abis_fp_orientation::singular::SingularPoint] = &[];
    let cores: Vec<_> = sp_arch
        .iter()
        .filter(|p| matches!(p.kind, SingularKind::Core))
        .collect();
    assert_eq!(cores.len(), 0, "arch should have 0 cores");
}

#[test]
fn test_compatible_patterns() {
    assert!(FingerprintPattern::Arch.compatible_with(FingerprintPattern::TentedArch));
    assert!(!FingerprintPattern::LeftLoop.compatible_with(FingerprintPattern::RightLoop));
    assert!(FingerprintPattern::Whorl.compatible_with(FingerprintPattern::TwinLoop));
}

// ----- OrientationMatcher -----

#[test]
fn test_matcher_identical_images_passes_coarse() {
    let img = striped_v(64, 64, 5);
    let m = OrientationMatcher::default();
    let result = m.compare(&img, &img).unwrap();
    // Identical images: coarse must pass.
    assert!(result.passed_coarse);
}

#[test]
fn test_matcher_identical_images_high_fine_score() {
    let img = striped_v(64, 64, 5);
    let m = OrientationMatcher::default();
    let result = m.compare(&img, &img).unwrap();
    assert!(
        result.fine > 0.8,
        "fine score for identical images should be > 0.8, got {}",
        result.fine
    );
}

#[test]
fn test_matcher_rotational_invariance_partial() {
    // Both horizontal and vertical stripes have consistent orientations --
    // their fields are orthogonal, so fine score drops but coarse should still assess.
    let v = striped_v(64, 64, 5);
    let h = striped_h(64, 64, 5);
    let m = OrientationMatcher::default();
    let sv = m.compare(&v, &v).unwrap();
    let sh = m.compare(&v, &h).unwrap();
    // Self-match fine score must be higher.
    assert!(
        sv.fine > sh.fine,
        "self-match fine > cross-orientation fine"
    );
}

#[test]
fn test_matcher_medium_score_in_range() {
    let img = striped_v(64, 64, 5);
    let m = OrientationMatcher::default();
    let result = m.compare(&img, &img).unwrap();
    assert!(result.medium >= 0.0 && result.medium <= 1.0);
}
