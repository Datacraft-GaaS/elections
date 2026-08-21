//! Tests for multi-finger slap segmentation, scoring, and DPR.

use eel_abis_fp_multifinger::{
    dpr::DprPolicy,
    matcher::MultiFingerMatcher,
    multifinger::{fuse_per_finger, position_weight, PerFingerScore},
    slap::{otsu_threshold, segment_slap, FingerPosition, SlapKind},
    GrayImage,
};

// Helper: create a slap-like image with N evenly-spaced dark bands (finger images).
fn slap_image(total_width: u32, height: u32, n_fingers: usize) -> GrayImage {
    let finger_w = total_width / n_fingers as u32;
    let gap_w = (total_width / (n_fingers as u32 * 8)).max(2);
    let pixels: Vec<u8> = (0..(total_width * height))
        .map(|i| {
            let x = i % total_width;
            let col = x / finger_w;
            let within_finger = x % finger_w;
            if col < n_fingers as u32 && within_finger >= gap_w && within_finger < finger_w - gap_w
            {
                50u8 // dark = fingerprint
            } else {
                220u8 // light = background
            }
        })
        .collect();
    GrayImage::new(total_width, height, pixels).unwrap()
}

fn make_score(pos: FingerPosition, score: f32, threshold: f32) -> PerFingerScore {
    PerFingerScore {
        position: pos,
        score,
        is_match: score >= threshold,
    }
}

// ----- GrayImage -----

#[test]
fn test_gray_image_new_ok() {
    assert!(GrayImage::new(32, 32, vec![128u8; 32 * 32]).is_ok());
}

#[test]
fn test_gray_image_crop_correct_size() {
    let img = GrayImage::new(100, 100, vec![0u8; 10000]).unwrap();
    let bb = eel_abis_fp_multifinger::slap::BoundingBox {
        x: 10,
        y: 10,
        width: 30,
        height: 40,
    };
    let cropped = img.crop(&bb);
    assert_eq!(cropped.width, 30);
    assert_eq!(cropped.height, 40);
}

// ----- Otsu threshold -----

#[test]
fn test_otsu_bimodal_image() {
    let mut pixels = vec![50u8; 500];
    pixels.extend_from_slice(&vec![200u8; 500]);
    let t = otsu_threshold(&pixels);
    // Threshold should fall between the two peaks.
    assert!(
        t >= 50 && t < 200,
        "Otsu threshold={t} should be between 50 and 200 for bimodal input"
    );
}

#[test]
fn test_otsu_uniform_image_returns_value() {
    let pixels = vec![128u8; 1000];
    let t = otsu_threshold(&pixels);
    // No inter-class variance; any value is OK as long as no panic.
    let _ = t;
}

// ----- Slap segmentation -----

#[test]
fn test_segment_right_four_produces_4_segments() {
    let img = slap_image(400, 100, 4);
    let segments = segment_slap(&img, SlapKind::RightFour).unwrap();
    assert_eq!(
        segments.len(),
        4,
        "right-four slap must segment into 4 fingers"
    );
}

#[test]
fn test_segment_thumbs_produces_2_segments() {
    let img = slap_image(200, 100, 2);
    let segments = segment_slap(&img, SlapKind::Thumbs).unwrap();
    assert_eq!(segments.len(), 2, "thumbs slap must segment into 2 fingers");
}

#[test]
fn test_segment_left_four_produces_4_segments() {
    let img = slap_image(400, 100, 4);
    let segments = segment_slap(&img, SlapKind::LeftFour).unwrap();
    assert_eq!(
        segments.len(),
        4,
        "left-four slap must segment into 4 fingers"
    );
}

#[test]
fn test_segment_positions_correct_for_right_four() {
    let img = slap_image(400, 100, 4);
    let segs = segment_slap(&img, SlapKind::RightFour).unwrap();
    assert_eq!(segs[0].0, FingerPosition::RightIndex);
    assert_eq!(segs[1].0, FingerPosition::RightMiddle);
    assert_eq!(segs[2].0, FingerPosition::RightRing);
    assert_eq!(segs[3].0, FingerPosition::RightLittle);
}

#[test]
fn test_segment_bounding_boxes_non_overlapping() {
    let img = slap_image(400, 100, 4);
    let segs = segment_slap(&img, SlapKind::RightFour).unwrap();
    for i in 0..segs.len() - 1 {
        let bb_a = segs[i].1;
        let bb_b = segs[i + 1].1;
        assert!(
            bb_a.x + bb_a.width <= bb_b.x + 1,
            "segment {i} and {} overlap: {:?} {:?}",
            i + 1,
            bb_a,
            bb_b
        );
    }
}

#[test]
fn test_segment_crops_have_correct_height() {
    let img = slap_image(400, 100, 4);
    let segs = segment_slap(&img, SlapKind::RightFour).unwrap();
    for (_, bb, crop) in &segs {
        assert_eq!(crop.height, 100, "cropped height must equal slap height");
    }
}

#[test]
fn test_segment_total_width_covers_image() {
    let img = slap_image(400, 100, 4);
    let segs = segment_slap(&img, SlapKind::RightFour).unwrap();
    let total_w: u32 = segs.iter().map(|(_, bb, _)| bb.width).sum();
    assert_eq!(
        total_w, 400,
        "sum of segment widths must equal image width, got {total_w}"
    );
}

#[test]
fn test_segment_too_small_image_errors() {
    let img = GrayImage::new(4, 4, vec![0u8; 16]).unwrap();
    assert!(segment_slap(&img, SlapKind::RightFour).is_err());
}

// ----- Position weights -----

#[test]
fn test_thumb_weight_highest() {
    assert!(
        position_weight(FingerPosition::RightThumb) > position_weight(FingerPosition::RightIndex),
        "thumb weight must exceed index weight"
    );
}

#[test]
fn test_index_weight_above_middle() {
    assert!(
        position_weight(FingerPosition::RightIndex) > position_weight(FingerPosition::RightMiddle)
    );
}

#[test]
fn test_little_weight_lowest() {
    assert!(
        position_weight(FingerPosition::RightLittle) < position_weight(FingerPosition::RightRing)
    );
}

// ----- Fusion -----

#[test]
fn test_fuse_per_finger_weighted_average() {
    let scores = vec![
        make_score(FingerPosition::RightThumb, 1.0, 0.4), // weight 1.2
        make_score(FingerPosition::RightLittle, 0.0, 0.4), // weight 0.8
    ];
    let fused = fuse_per_finger(&scores);
    // Weighted avg = (1.2 * 1.0 + 0.8 * 0.0) / (1.2 + 0.8) = 0.6
    assert!((fused - 0.6).abs() < 1e-4, "expected 0.6, got {fused}");
}

#[test]
fn test_fuse_empty_scores_returns_zero() {
    assert_eq!(fuse_per_finger(&[]), 0.0);
}

// ----- DPR -----

#[test]
fn test_dpr_rejects_when_0_of_6_match() {
    let policy = DprPolicy::default();
    let r = policy.evaluate(0, 6);
    assert!(!r.passed, "DPR must reject 0/6");
}

#[test]
fn test_dpr_rejects_when_1_of_6_match() {
    let policy = DprPolicy::default();
    let r = policy.evaluate(1, 6);
    assert!(!r.passed, "DPR must reject 1/6");
}

#[test]
fn test_dpr_passes_when_2_of_6_match() {
    let policy = DprPolicy::default();
    let r = policy.evaluate(2, 6);
    assert!(r.passed, "DPR must pass 2/6");
}

#[test]
fn test_dpr_passes_when_fewer_than_min_available() {
    // Gate only activates when >= min_available fingers are present.
    let policy = DprPolicy::default();
    let r = policy.evaluate(0, 3); // only 3 fingers available
    assert!(
        r.passed,
        "DPR must pass when available < min_available (3 < 6)"
    );
}

#[test]
fn test_dpr_reason_present_on_rejection() {
    let policy = DprPolicy::default();
    let r = policy.evaluate(0, 6);
    assert!(r.reason.is_some());
}

// ----- MultiFingerMatcher -----

#[test]
fn test_matcher_identical_slap_scores_above_zero() {
    let img = slap_image(400, 100, 4);
    let m = MultiFingerMatcher::default();
    let result = m.compare_slaps(&img, &img, SlapKind::RightFour).unwrap();
    assert!(
        result.overall_score > 0.0,
        "identical slap comparison must score > 0"
    );
}

#[test]
fn test_matcher_count_matched_for_identical() {
    let img = slap_image(400, 100, 4);
    let m = MultiFingerMatcher::default();
    let result = m.compare_slaps(&img, &img, SlapKind::RightFour).unwrap();
    assert!(
        result.count_matched > 0,
        "identical slap should match at least 1 finger"
    );
}
