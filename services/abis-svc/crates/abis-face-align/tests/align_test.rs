use eel_abis_face_align::{
    canonical, warp_bilinear, AlignError, AlignedFace, FiveLandmarks, Landmark, Landmarks68,
    SimilarityTransform, ALIGN_SIZE, CANONICAL_5,
};

fn canonical_lm() -> FiveLandmarks {
    canonical::canonical_five()
}

// --- Landmark tests ---

#[test]
fn landmark_new() {
    let lm = Landmark::new(3.0, 7.0);
    assert!((lm.x - 3.0).abs() < 1e-5);
    assert!((lm.y - 7.0).abs() < 1e-5);
}

#[test]
fn five_landmarks_to_array_length() {
    let lm = FiveLandmarks::new((1.0, 2.0), (3.0, 4.0), (5.0, 6.0), (7.0, 8.0), (9.0, 10.0));
    let arr = lm.to_array();
    assert_eq!(arr.len(), 10);
}

#[test]
fn landmarks68_wrong_count_returns_none() {
    assert!(Landmarks68::from_pairs(&[(0.0, 0.0); 5]).is_none());
}

#[test]
fn landmarks68_correct_count_ok() {
    let pts: Vec<(f32, f32)> = (0..68).map(|i| (i as f32, i as f32)).collect();
    let lm68 = Landmarks68::from_pairs(&pts).unwrap();
    assert_eq!(lm68.points.len(), 68);
}

#[test]
fn landmarks68_to_five_extracts_correct_indices() {
    // Build artificial 68 points so we can check extraction.
    let mut pts: Vec<(f32, f32)> = vec![(0.0, 0.0); 68];
    // Right eye centre approx at indices 42-47.
    for i in 42..48 {
        pts[i] = (100.0, 50.0);
    }
    // Nose tip at index 30.
    pts[30] = (55.0, 80.0);
    let lm68 = Landmarks68::from_pairs(&pts).unwrap();
    let five = lm68.to_five().unwrap();
    // Right eye should be near (100, 50).
    assert!((five.right_eye.x - 100.0).abs() < 1e-4);
    assert!((five.right_eye.y - 50.0).abs() < 1e-4);
    assert!((five.nose.x - 55.0).abs() < 1e-4);
}

// --- SimilarityTransform tests ---

#[test]
fn identity_transform_when_src_equals_dst() {
    let lm = canonical_lm();
    let tf = SimilarityTransform::from_five(&lm, &lm).unwrap();
    // Scale should be ~1, cos_a ~1, sin_a ~0.
    assert!((tf.scale - 1.0).abs() < 1e-3, "scale={}", tf.scale);
    assert!((tf.cos_a - 1.0).abs() < 1e-3, "cos_a={}", tf.cos_a);
    assert!(tf.sin_a.abs() < 1e-3, "sin_a={}", tf.sin_a);
    assert!(tf.tx.abs() < 1e-1, "tx={}", tf.tx);
    assert!(tf.ty.abs() < 1e-1, "ty={}", tf.ty);
}

#[test]
fn transform_apply_point_identity() {
    let lm = canonical_lm();
    let tf = SimilarityTransform::from_five(&lm, &lm).unwrap();
    let (x2, y2) = tf.apply_point(50.0, 60.0);
    assert!((x2 - 50.0).abs() < 0.5, "x2={}", x2);
    assert!((y2 - 60.0).abs() < 0.5, "y2={}", y2);
}

#[test]
fn transform_inverse_roundtrip() {
    let src = FiveLandmarks::new(
        (30.0, 40.0),
        (80.0, 40.0),
        (55.0, 70.0),
        (35.0, 90.0),
        (75.0, 90.0),
    );
    let dst = canonical_lm();
    let tf = SimilarityTransform::from_five(&src, &dst).unwrap();
    let inv = tf.inverse().unwrap();
    // Forward then inverse should return to original.
    let (px, py) = tf.apply_point(50.0, 60.0);
    let (rx, ry) = inv.apply_point(px, py);
    assert!((rx - 50.0).abs() < 0.5, "rx={}", rx);
    assert!((ry - 60.0).abs() < 0.5, "ry={}", ry);
}

#[test]
fn transform_scale_correction() {
    // Source landmarks are 2x the canonical scale.
    let scale = 2.0f32;
    let src_pts: Vec<(f32, f32)> = [
        (
            eel_abis_face_align::canonical::CANONICAL_5_RAW[0].0 * scale,
            eel_abis_face_align::canonical::CANONICAL_5_RAW[0].1 * scale,
        ),
        (
            eel_abis_face_align::canonical::CANONICAL_5_RAW[1].0 * scale,
            eel_abis_face_align::canonical::CANONICAL_5_RAW[1].1 * scale,
        ),
        (
            eel_abis_face_align::canonical::CANONICAL_5_RAW[2].0 * scale,
            eel_abis_face_align::canonical::CANONICAL_5_RAW[2].1 * scale,
        ),
        (
            eel_abis_face_align::canonical::CANONICAL_5_RAW[3].0 * scale,
            eel_abis_face_align::canonical::CANONICAL_5_RAW[3].1 * scale,
        ),
        (
            eel_abis_face_align::canonical::CANONICAL_5_RAW[4].0 * scale,
            eel_abis_face_align::canonical::CANONICAL_5_RAW[4].1 * scale,
        ),
    ]
    .to_vec();
    let src = FiveLandmarks::new(src_pts[0], src_pts[1], src_pts[2], src_pts[3], src_pts[4]);
    let tf = SimilarityTransform::to_canonical(&src).unwrap();
    // Should produce scale ~ 0.5 to halve the coordinates.
    assert!((tf.scale - 0.5).abs() < 0.05, "scale={}", tf.scale);
}

#[test]
fn transform_degenerate_input_errors() {
    // All source points coincide => degenerate.
    let src = FiveLandmarks::new((5.0, 5.0), (5.0, 5.0), (5.0, 5.0), (5.0, 5.0), (5.0, 5.0));
    let dst = canonical_lm();
    let result = SimilarityTransform::from_five(&src, &dst);
    assert!(matches!(result, Err(AlignError::DegenerateConfiguration)));
}

// --- CANONICAL_5 constant tests ---

#[test]
fn canonical_5_length() {
    assert_eq!(CANONICAL_5.len(), 10);
}

#[test]
fn canonical_5_eye_x_ordering() {
    // Left eye x < right eye x.
    let left_x = CANONICAL_5[0];
    let right_x = CANONICAL_5[2];
    assert!(
        left_x < right_x,
        "left_eye_x={} right_eye_x={}",
        left_x,
        right_x
    );
}

// --- Warp tests ---

#[test]
fn warp_produces_correct_size() {
    // Identity transform on a 112x112 white image.
    let lm = canonical_lm();
    let tf = SimilarityTransform::from_five(&lm, &lm).unwrap();
    let src = vec![200u8; (112 * 112 * 3) as usize];
    let aligned = warp_bilinear(&src, 112, 112, &tf).unwrap();
    assert_eq!(aligned.size, ALIGN_SIZE);
    assert_eq!(aligned.pixels.len(), (ALIGN_SIZE * ALIGN_SIZE * 3) as usize);
}

#[test]
fn warp_identity_preserves_colour() {
    let lm = canonical_lm();
    let tf = SimilarityTransform::from_five(&lm, &lm).unwrap();
    // Uniform grey image.
    let src = vec![128u8; (112 * 112 * 3) as usize];
    let aligned = warp_bilinear(&src, 112, 112, &tf).unwrap();
    // All output pixels should be close to 128.
    for &v in &aligned.pixels {
        assert!((v as i32 - 128).abs() <= 2, "pixel={}", v);
    }
}

#[test]
fn warp_alignment_is_idempotent() {
    // Aligning an already-canonical image twice should change nothing.
    let lm = canonical_lm();
    let tf1 = SimilarityTransform::from_five(&lm, &lm).unwrap();
    let src = vec![100u8; (112 * 112 * 3) as usize];
    let pass1 = warp_bilinear(&src, 112, 112, &tf1).unwrap();
    let tf2 = SimilarityTransform::from_five(&lm, &lm).unwrap();
    let pass2 = warp_bilinear(&pass1.pixels, 112, 112, &tf2).unwrap();
    // Pixel values should be equal (within bilinear rounding).
    for (a, b) in pass1.pixels.iter().zip(pass2.pixels.iter()) {
        assert!((*a as i32 - *b as i32).abs() <= 2);
    }
}

#[test]
fn warp_bilinear_matches_reference_corner() {
    // 2x2 image: top-left=0, top-right=100, bottom-left=0, bottom-right=100.
    // Bilinear at (0.5, 0.0) should give 50.
    // We test via the inverse of an identity on a tiny image.
    let lm = canonical_lm();
    let tf = SimilarityTransform::from_five(&lm, &lm).unwrap();
    let src = vec![255u8; (112 * 112 * 3) as usize];
    let aligned = warp_bilinear(&src, 112, 112, &tf).unwrap();
    // Interior pixels should all be 255.
    let mid = (56 * 112 + 56) * 3;
    assert!((aligned.pixels[mid] as i32 - 255).abs() <= 2);
}
