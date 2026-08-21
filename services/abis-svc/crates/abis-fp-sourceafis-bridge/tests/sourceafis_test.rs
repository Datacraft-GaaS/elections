//! Tests for the SourceAFIS bridge crate.
//!
//! Uses synthetic minutia corpora and small synthetic images so no real
//! fingerprint images are required.

use eel_abis_fp_sourceafis_bridge::{
    matcher::SourceAfisMatcher,
    minutia::{MinutiaType, SourceAfisMinutia},
    params::SourceAfisParams,
    ridge_structure::{build_edge_table, edge_similarity},
    skeleton::{extract_skeleton, Skeleton},
    GrayImage, SourceAfisError,
};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a tiny all-white (background) image with a single ridge pixel at (cx, cy).
fn solid_image(w: u32, h: u32, value: u8) -> GrayImage {
    GrayImage::new(vec![value; (w * h) as usize], w, h)
}

/// Synthetic image: horizontal stripe of dark pixels (ridge) across the middle.
fn stripe_image(w: u32, h: u32) -> GrayImage {
    let mut pixels = vec![200u8; (w * h) as usize];
    let mid = h / 2;
    for x in 0..w {
        pixels[(mid * w + x) as usize] = 30;
    }
    GrayImage::new(pixels, w, h)
}

/// Create a small test minutia at position (x, y) with direction `dir`.
fn minutia(x: f32, y: f32, dir: f32) -> SourceAfisMinutia {
    SourceAfisMinutia {
        x,
        y,
        direction: dir,
        minutia_type: MinutiaType::Ending,
    }
}

/// Build a synthetic minutia set arranged in a regular grid pattern.
fn grid_minutiae(n: usize) -> Vec<SourceAfisMinutia> {
    (0..n)
        .map(|i| {
            minutia(
                10.0 * i as f32,
                10.0 * (i % 3) as f32,
                PI * i as f32 / n as f32,
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// GrayImage
// ---------------------------------------------------------------------------

#[test]
fn gray_image_pixel_accessor() {
    let img = GrayImage::new(vec![10, 20, 30, 40], 2, 2);
    assert_eq!(img.pixel(0, 0), 10);
    assert_eq!(img.pixel(1, 1), 40);
}

#[test]
fn gray_image_size() {
    let img = solid_image(32, 32, 128);
    assert_eq!(img.pixels.len(), 1024);
}

// ---------------------------------------------------------------------------
// Skeleton extraction
// ---------------------------------------------------------------------------

#[test]
fn skeleton_rejects_tiny_image() {
    let img = solid_image(8, 8, 100);
    match extract_skeleton(&img) {
        Err(SourceAfisError::ImageTooSmall(8, 8)) => {}
        other => panic!("expected ImageTooSmall, got {other:?}"),
    }
}

#[test]
fn skeleton_accepts_min_size() {
    let img = solid_image(16, 16, 200);
    assert!(extract_skeleton(&img).is_ok());
}

#[test]
fn skeleton_same_dimensions_as_input() {
    let img = solid_image(32, 40, 200);
    let skel = extract_skeleton(&img).unwrap();
    assert_eq!(skel.width, 32);
    assert_eq!(skel.height, 40);
}

#[test]
fn all_background_skeleton_is_empty() {
    // All-white image: no ridge pixels after binarisation.
    let img = solid_image(32, 32, 255);
    let skel = extract_skeleton(&img).unwrap();
    assert!(skel.pixels.iter().all(|&p| !p), "expected no ridge pixels");
}

#[test]
fn stripe_skeleton_not_empty() {
    let img = stripe_image(32, 32);
    let skel = extract_skeleton(&img).unwrap();
    // After thinning a horizontal stripe, some ridge pixels remain.
    let count = skel.pixels.iter().filter(|&&p| p).count();
    assert!(count > 0, "expected some ridge pixels from stripe");
}

// ---------------------------------------------------------------------------
// Local ridge structure
// ---------------------------------------------------------------------------

#[test]
fn edge_table_length_matches_minutia_count() {
    let mins = grid_minutiae(6);
    let params = SourceAfisParams::default();
    let table = build_edge_table(&mins, &params);
    assert_eq!(table.len(), 6);
}

#[test]
fn edge_table_no_self_edges() {
    let mins = grid_minutiae(5);
    let params = SourceAfisParams::default();
    let table = build_edge_table(&mins, &params);
    for (i, edges) in table.iter().enumerate() {
        assert!(
            edges.iter().all(|e| e.neighbour_idx != i),
            "self-edge at {i}"
        );
    }
}

#[test]
fn descriptor_rotation_invariant() {
    // Rotate the whole minutia set by PI/3 and verify edge distances are unchanged.
    let rot = PI / 3.0;
    let mins_a: Vec<SourceAfisMinutia> = (0..4)
        .map(|i| minutia(i as f32 * 50.0, i as f32 * 20.0, 0.0))
        .collect();
    let mins_b: Vec<SourceAfisMinutia> = mins_a
        .iter()
        .map(|m| {
            let x = m.x * rot.cos() - m.y * rot.sin();
            let y = m.x * rot.sin() + m.y * rot.cos();
            minutia(x, y, m.direction + rot)
        })
        .collect();
    let params = SourceAfisParams::default();
    let ta = build_edge_table(&mins_a, &params);
    let tb = build_edge_table(&mins_b, &params);
    // Edge counts must match (same topology).
    assert_eq!(ta[0].len(), tb[0].len());
    // Distances must be invariant.
    for (ea, eb) in ta[0].iter().zip(tb[0].iter()) {
        assert!((ea.distance - eb.distance).abs() < 0.5);
    }
}

#[test]
fn edge_similarity_identical_edges() {
    let mins = grid_minutiae(4);
    let params = SourceAfisParams::default();
    let table = build_edge_table(&mins, &params);
    if let (Some(e0), Some(e1)) = (table[0].first(), table[0].first()) {
        let s = edge_similarity(e0, e1, &params, params.neighbour_radius_px);
        // Identical edges should produce Some with score near 1.
        assert!(s.is_some());
        assert!(s.unwrap() > 0.9);
    }
}

// ---------------------------------------------------------------------------
// Matcher
// ---------------------------------------------------------------------------

#[test]
fn score_bounded_zero_to_one() {
    let probe = grid_minutiae(8);
    let gallery = grid_minutiae(8);
    let matcher = SourceAfisMatcher::default();
    let score = matcher.compare_minutiae(&probe, &gallery);
    let c = score.combined();
    assert!((0.0..=1.0).contains(&c), "score {c} out of range");
}

#[test]
fn identical_minutiae_score_high() {
    let mins = grid_minutiae(12);
    let matcher = SourceAfisMatcher::default();
    let score = matcher.compare_minutiae(&mins, &mins);
    // With 12 paired minutiae: raw ≈ 12-14, combined = raw/(raw+40) ≈ 0.23.
    // Threshold set to 0.15 to account for small synthetic corpus size.
    assert!(
        score.combined() > 0.15,
        "identical minutiae should score above 0.15, got {}",
        score.combined()
    );
}

#[test]
fn empty_probe_scores_zero() {
    let gallery = grid_minutiae(6);
    let matcher = SourceAfisMatcher::default();
    let score = matcher.compare_minutiae(&[], &gallery);
    assert_eq!(score.combined(), 0.0);
}

#[test]
fn empty_gallery_scores_zero() {
    let probe = grid_minutiae(6);
    let matcher = SourceAfisMatcher::default();
    let score = matcher.compare_minutiae(&probe, &[]);
    assert_eq!(score.combined(), 0.0);
}

#[test]
fn extension_terminates_for_large_sets() {
    // Large synthetic sets: extension must terminate without infinite loops.
    let probe = grid_minutiae(50);
    let gallery = grid_minutiae(50);
    let matcher = SourceAfisMatcher::default();
    let score = matcher.compare_minutiae(&probe, &gallery);
    let _ = score.combined();
}

#[test]
fn compare_image_api_returns_score() {
    let img = stripe_image(64, 64);
    let matcher = SourceAfisMatcher::default();
    let score = matcher.compare(&img, &img).unwrap();
    assert!((0.0..=1.0).contains(&score));
}
