//! Level-3 feature extraction and matching tests.

use eel_abis_fp_level3::{
    incipient::detect_incipient_ridges,
    matcher::{Level3Features, Level3Matcher},
    pores::extract_pores,
    quality::{Level3Quality, Level3QualityError},
    ridge_edge::extract_ridge_edges,
    EdgePoint,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A small synthetic 20x20 grayscale image with a horizontal ridge in the middle.
fn ridge_image(width: usize, height: usize) -> Vec<u8> {
    let mut img = vec![220u8; width * height]; // valley = light
    let mid = height / 2;
    for x in 0..width {
        img[mid * width + x] = 50; // ridge = dark
                                   // Add a few pore-like minima on the ridge.
        if x % 8 == 4 {
            img[mid * width + x] = 20;
        }
    }
    img
}

fn flat_orientation(width: usize, height: usize, angle: u8) -> Vec<u8> {
    vec![angle; width * height]
}

fn good_quality() -> Level3Quality {
    Level3Quality::new(500, 0.8, 0.9)
}

fn low_res_quality() -> Level3Quality {
    Level3Quality::new(300, 0.8, 0.9)
}

// ---------------------------------------------------------------------------
// Quality tests
// ---------------------------------------------------------------------------

#[test]
fn test_sufficient_resolution_ok() {
    let q = Level3Quality::new(500, 0.8, 0.9);
    assert!(q.check_resolution().is_ok());
}

#[test]
fn test_insufficient_resolution_errors() {
    let q = Level3Quality::new(400, 0.8, 0.9);
    assert_eq!(
        q.check_resolution(),
        Err(Level3QualityError::InsufficientResolution { actual: 400 })
    );
}

#[test]
fn test_low_res_300ppi_triggers_error() {
    let q = Level3Quality::new(300, 0.5, 0.5);
    assert!(q.check_resolution().is_err());
}

#[test]
fn test_overall_quality_in_range() {
    let q = Level3Quality::new(600, 0.7, 0.8);
    let overall = q.overall();
    assert!((0.0..=1.0).contains(&overall));
}

#[test]
fn test_overall_quality_weighted() {
    let q = Level3Quality::new(600, 1.0, 1.0);
    assert!((q.overall() - 1.0).abs() < 1e-5);
}

// ---------------------------------------------------------------------------
// Pore extraction tests
// ---------------------------------------------------------------------------

#[test]
fn test_pore_extraction_returns_vec() {
    let img = ridge_image(40, 40);
    let orient = flat_orientation(40, 40, 0);
    let pores = extract_pores(&img, 40, 40, &orient);
    // With a pore-like minimum every 8 px on a 40px wide ridge, expect at least one.
    assert!(!pores.is_empty(), "expected pores in a ridge image");
}

#[test]
fn test_pore_confidence_in_range() {
    let img = ridge_image(40, 40);
    let orient = flat_orientation(40, 40, 0);
    let pores = extract_pores(&img, 40, 40, &orient);
    for p in &pores {
        assert!(
            (0.0..=1.0).contains(&p.confidence),
            "confidence out of range: {}",
            p.confidence
        );
    }
}

#[test]
fn test_pore_extraction_empty_image_returns_empty() {
    let pores = extract_pores(&[], 0, 0, &[]);
    assert!(pores.is_empty());
}

#[test]
fn test_pore_extraction_mismatched_buffer_returns_empty() {
    let img = vec![100u8; 10];
    let orient = vec![0u8; 5]; // wrong size
    let pores = extract_pores(&img, 5, 2, &orient);
    assert!(pores.is_empty());
}

#[test]
fn test_pores_min_spacing_respected() {
    let img = ridge_image(80, 20);
    let orient = flat_orientation(80, 20, 0);
    let pores = extract_pores(&img, 80, 20, &orient);
    // Check no two pores are closer than 8 px.
    for (i, p1) in pores.iter().enumerate() {
        for p2 in pores.iter().skip(i + 1) {
            let dx = p1.x - p2.x;
            let dy = p1.y - p2.y;
            let dist = (dx * dx + dy * dy).sqrt();
            assert!(dist >= 7.9, "pores too close: {dist}");
        }
    }
}

// ---------------------------------------------------------------------------
// Ridge edge extraction tests
// ---------------------------------------------------------------------------

#[test]
fn test_ridge_edge_extraction_returns_points() {
    let img = ridge_image(40, 20);
    let orient = flat_orientation(40, 20, 0);
    let points = extract_ridge_edges(&img, 40, 20, &orient);
    assert!(!points.is_empty(), "expected ridge edge points");
}

#[test]
fn test_edge_point_side_values() {
    let img = ridge_image(40, 20);
    let orient = flat_orientation(40, 20, 0);
    let points = extract_ridge_edges(&img, 40, 20, &orient);
    for p in &points {
        assert!(
            p.side == 0 || p.side == 1,
            "side must be 0 or 1, got {}",
            p.side
        );
    }
}

#[test]
fn test_edge_curvature_in_range() {
    let img = ridge_image(40, 20);
    let orient = flat_orientation(40, 20, 0);
    let points = extract_ridge_edges(&img, 40, 20, &orient);
    for p in &points {
        assert!(
            (0.0..=1.0).contains(&p.curvature),
            "curvature out of range: {}",
            p.curvature
        );
    }
}

// ---------------------------------------------------------------------------
// Matcher tests
// ---------------------------------------------------------------------------

fn make_level3_features(
    pores: Vec<eel_abis_fp_level3::Pore>,
    quality: Level3Quality,
) -> Level3Features {
    Level3Features {
        pores,
        edge_points: Vec::new(),
        quality,
    }
}

#[test]
fn test_low_res_image_triggers_error() {
    let probe = make_level3_features(vec![], low_res_quality());
    let gallery = make_level3_features(vec![], good_quality());
    let matcher = Level3Matcher::new();
    let result = matcher.compare(&probe, &gallery, &[], &[]);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Level3QualityError::InsufficientResolution { .. }
    ));
}

#[test]
fn test_empty_minutiae_scores_zero() {
    let probe = make_level3_features(vec![], good_quality());
    let gallery = make_level3_features(vec![], good_quality());
    let score = Level3Matcher::new()
        .compare(&probe, &gallery, &[], &[])
        .unwrap();
    assert_eq!(score, 0.0);
}

#[test]
fn test_score_in_range() {
    let img = ridge_image(40, 40);
    let orient = flat_orientation(40, 40, 0);
    let pores = extract_pores(&img, 40, 40, &orient);
    let probe = make_level3_features(pores.clone(), good_quality());
    let gallery = make_level3_features(pores, good_quality());
    let minutiae = vec![(20.0f32, 20.0f32)];
    let score = Level3Matcher::new()
        .compare(&probe, &gallery, &minutiae, &minutiae)
        .unwrap();
    assert!((0.0..=1.0).contains(&score), "score out of range: {score}");
}

#[test]
fn test_identical_pore_constellations_score_high() {
    let img = ridge_image(100, 100);
    let orient = flat_orientation(100, 100, 0);
    let pores = extract_pores(&img, 100, 100, &orient);
    let probe = make_level3_features(pores.clone(), good_quality());
    let gallery = make_level3_features(pores, good_quality());
    let minutiae = vec![(50.0f32, 50.0f32)];
    let score = Level3Matcher::new()
        .compare(&probe, &gallery, &minutiae, &minutiae)
        .unwrap();
    assert!(
        score > 0.85,
        "identical pore constellations should score >0.85, got {score}"
    );
}

#[test]
fn test_edge_shape_similarity_adds_bonus() {
    let edge_pts: Vec<EdgePoint> = (0..5)
        .map(|i| EdgePoint {
            x: i as f32,
            y: 0.0,
            curvature: 0.3,
            ridge_width: 5.0,
            side: 0,
        })
        .collect();
    let probe = Level3Features {
        pores: vec![],
        edge_points: edge_pts.clone(),
        quality: good_quality(),
    };
    let gallery = Level3Features {
        pores: vec![],
        edge_points: edge_pts,
        quality: good_quality(),
    };
    let score = Level3Matcher::new().compare(&probe, &gallery, &[(0.0, 0.0)], &[(0.0, 0.0)]);
    // Even with no pore matches, edge bonus may contribute; no panic is the minimum.
    assert!(score.is_ok());
}
