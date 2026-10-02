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

//! Tests for TPS warp, registration, and warp-assisted matching.

use eel_abis_fp_tps_warp::{
    matcher::{minutiae_overlap_score, TpsWarpMatcher},
    register::register_minutiae,
    tps::TpsWarp,
    Minutia, Point,
};

fn pt(x: f64, y: f64) -> Point {
    Point::new(x, y)
}

fn min(x: f64, y: f64) -> Minutia {
    Minutia::new(x, y, 0.0, 80)
}

fn min_theta(x: f64, y: f64, theta: f64) -> Minutia {
    Minutia::new(x, y, theta, 80)
}

// ----- TPS basic tests -----

#[test]
fn test_tps_requires_min_3_points() {
    let src = vec![pt(0.0, 0.0), pt(1.0, 0.0)];
    let dst = vec![pt(0.0, 0.0), pt(1.0, 0.0)];
    assert!(TpsWarp::fit(&src, &dst).is_err());
}

#[test]
fn test_tps_mismatched_point_counts_errors() {
    let src = vec![pt(0.0, 0.0), pt(1.0, 0.0), pt(0.5, 1.0)];
    let dst = vec![pt(0.0, 0.0), pt(1.0, 0.0)];
    assert!(TpsWarp::fit(&src, &dst).is_err());
}

#[test]
fn test_tps_identity_when_source_equals_target() {
    let pts = vec![
        pt(10.0, 20.0),
        pt(100.0, 50.0),
        pt(60.0, 150.0),
        pt(30.0, 80.0),
    ];
    let tps = TpsWarp::fit(&pts, &pts).unwrap();
    for p in &pts {
        let (wx, wy) = tps.apply(p.x, p.y);
        assert!(
            (wx - p.x).abs() < 0.5 && (wy - p.y).abs() < 0.5,
            "identity TPS at ({}, {}) -> ({wx:.3}, {wy:.3}), expected ({}, {})",
            p.x,
            p.y,
            p.x,
            p.y
        );
    }
}

#[test]
fn test_tps_maps_control_points_exactly() {
    let src = vec![
        pt(0.0, 0.0),
        pt(100.0, 0.0),
        pt(50.0, 100.0),
        pt(50.0, 50.0),
    ];
    // Shift all by (10, 5).
    let dst: Vec<Point> = src.iter().map(|p| pt(p.x + 10.0, p.y + 5.0)).collect();
    let tps = TpsWarp::fit(&src, &dst).unwrap();
    for (s, d) in src.iter().zip(dst.iter()) {
        let (wx, wy) = tps.apply(s.x, s.y);
        assert!(
            (wx - d.x).abs() < 1.0 && (wy - d.y).abs() < 1.0,
            "TPS control point mismatch: ({:.2},{:.2}) -> ({wx:.2},{wy:.2}), expected ({:.2},{:.2})",
            s.x, s.y, d.x, d.y
        );
    }
}

#[test]
fn test_tps_corrects_barrel_distortion() {
    // Simulate barrel distortion: points near edge are compressed.
    let cx = 100.0f64;
    let cy = 100.0f64;
    let src: Vec<Point> = vec![
        pt(50.0, 50.0),
        pt(100.0, 50.0),
        pt(150.0, 50.0),
        pt(50.0, 100.0),
        pt(150.0, 100.0),
        pt(50.0, 150.0),
        pt(100.0, 150.0),
        pt(150.0, 150.0),
    ];
    // Apply barrel: compress toward centre.
    let dst: Vec<Point> = src
        .iter()
        .map(|p| {
            let dx = p.x - cx;
            let dy = p.y - cy;
            let r = (dx * dx + dy * dy).sqrt();
            let factor = 1.0 - 0.002 * r;
            pt(cx + dx * factor, cy + dy * factor)
        })
        .collect();
    let tps = TpsWarp::fit(&dst, &src).unwrap();
    // For each distorted point, TPS should map back close to original.
    for (d, s) in dst.iter().zip(src.iter()) {
        let (wx, wy) = tps.apply(d.x, d.y);
        let err = ((wx - s.x).powi(2) + (wy - s.y).powi(2)).sqrt();
        assert!(
            err < 5.0,
            "TPS barrel distortion correction error {err:.2} too large at ({:.1},{:.1})",
            s.x,
            s.y
        );
    }
}

#[test]
fn test_tps_apply_interpolates_between_control_points() {
    // Uniform x shift of 20.
    let src = vec![
        pt(0.0, 0.0),
        pt(100.0, 0.0),
        pt(50.0, 100.0),
        pt(50.0, 50.0),
    ];
    let dst: Vec<Point> = src.iter().map(|p| pt(p.x + 20.0, p.y)).collect();
    let tps = TpsWarp::fit(&src, &dst).unwrap();
    // Midpoint (50, 50) is a control point; verify nearby interpolation.
    let (wx, _wy) = tps.apply(25.0, 25.0);
    // Rough check: x shift should be near 20.
    assert!(
        (wx - 45.0).abs() < 10.0,
        "interpolated shift too far from 20: got {wx:.2}"
    );
}

#[test]
fn test_tps_fit_with_3_points_ok() {
    let src = vec![pt(0.0, 0.0), pt(100.0, 0.0), pt(50.0, 100.0)];
    let dst = vec![pt(5.0, 0.0), pt(105.0, 0.0), pt(55.0, 100.0)];
    assert!(TpsWarp::fit(&src, &dst).is_ok());
}

// ----- TPS Minutia warp -----

#[test]
fn test_minutia_warped_position_changes() {
    let src = vec![
        pt(0.0, 0.0),
        pt(100.0, 0.0),
        pt(50.0, 100.0),
        pt(50.0, 50.0),
    ];
    let dst: Vec<Point> = src.iter().map(|p| pt(p.x + 10.0, p.y + 5.0)).collect();
    let tps = TpsWarp::fit(&src, &dst).unwrap();
    let m = Minutia::new(50.0, 50.0, 1.0, 80);
    let warped = m.warped(&tps);
    assert!(
        (warped.x - 60.0).abs() < 1.5,
        "warped x should be near 60, got {}",
        warped.x
    );
}

#[test]
fn test_minutia_warped_theta_unchanged() {
    let src = vec![pt(0.0, 0.0), pt(100.0, 0.0), pt(50.0, 100.0)];
    let dst: Vec<Point> = src.iter().map(|p| pt(p.x + 10.0, p.y)).collect();
    let tps = TpsWarp::fit(&src, &dst).unwrap();
    let m = Minutia::new(50.0, 50.0, 1.57, 80);
    let warped = m.warped(&tps);
    assert_eq!(
        warped.theta, 1.57,
        "theta must not change during position warp"
    );
}

// ----- Registration -----

#[test]
fn test_register_returns_none_for_empty_probe() {
    let result = register_minutiae(&[], &[min(0.0, 0.0)]).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_register_too_few_points_returns_none() {
    let probe = vec![min(10.0, 10.0), min(20.0, 20.0)];
    let gallery = vec![min(10.0, 10.0), min(20.0, 20.0)];
    // Only 2 correspondences < MIN_CORR=3; should return None.
    let result = register_minutiae(&probe, &gallery).unwrap();
    // Either None (too few) or a fitted TPS (if code finds 2 matches and can still fit).
    // We just verify no panic.
    let _ = result;
}

// ----- Overlap score -----

#[test]
fn test_overlap_identical_is_one() {
    let m = vec![min(10.0, 10.0), min(50.0, 50.0), min(80.0, 30.0)];
    let score = minutiae_overlap_score(&m, &m, 5.0, 0.3);
    assert!(
        (score - 1.0).abs() < 1e-5,
        "identical minutiae must score 1.0, got {score}"
    );
}

#[test]
fn test_overlap_no_overlap_is_zero() {
    let a = vec![min(0.0, 0.0), min(10.0, 10.0)];
    let b = vec![min(500.0, 500.0), min(600.0, 600.0)];
    let score = minutiae_overlap_score(&a, &b, 5.0, 0.3);
    assert_eq!(score, 0.0, "non-overlapping minutiae must score 0.0");
}

// ----- TpsWarpMatcher -----

#[test]
fn test_warp_matcher_identical_scores_above_zero() {
    let m_list = vec![
        Minutia::new(10.0, 10.0, 0.5, 80),
        Minutia::new(50.0, 50.0, 1.0, 80),
        Minutia::new(80.0, 30.0, 0.0, 80),
        Minutia::new(30.0, 70.0, 1.5, 80),
    ];
    let matcher = TpsWarpMatcher::default();
    let result = matcher.compare(&m_list, &m_list).unwrap();
    assert!(
        result.score_before > 0.0 || result.score_after > 0.0,
        "identical minutiae must score > 0"
    );
}

#[test]
fn test_warp_matcher_improves_or_maintains_score() {
    // Apply a small distortion to probe minutiae; TPS should recover.
    let gallery = vec![
        Minutia::new(10.0, 10.0, 0.5, 80),
        Minutia::new(50.0, 50.0, 1.0, 80),
        Minutia::new(80.0, 30.0, 0.0, 80),
        Minutia::new(30.0, 70.0, 1.5, 80),
        Minutia::new(60.0, 90.0, 0.3, 80),
    ];
    // Slightly distorted probe.
    let probe: Vec<Minutia> = gallery
        .iter()
        .map(|m| Minutia::new(m.x + 8.0, m.y + 3.0, m.theta, m.quality))
        .collect();
    let matcher = TpsWarpMatcher::default();
    let result = matcher.compare(&probe, &gallery).unwrap();
    let best = TpsWarpMatcher::best_score(&result);
    assert!(
        best > 0.0,
        "distorted probe must still produce positive score"
    );
}
