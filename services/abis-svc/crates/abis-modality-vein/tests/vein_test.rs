use chrono::Utc;
use eel_abis_modality_vein::{
    matcher::{match_vein, ncc, DEFAULT_THRESHOLD},
    parser::{parse_nir_bitmap, serialize_nir_bitmap, VeinError},
    quality::{count_branch_points, score_quality},
    BodySide, VeinTemplate, VeinType,
};
fn make_template(w: u16, h: u16, fill: u8, vein_type: VeinType, side: BodySide) -> VeinTemplate {
    VeinTemplate {
        template_id: "test-id".to_string(),
        vein_type,
        body_side: side,
        nir_wavelength_nm: 850,
        width: w,
        height: h,
        pattern_bitmap: vec![fill; (w as usize) * (h as usize)],
        captured_at: Utc::now(),
    }
}

fn checkerboard(w: u16, h: u16) -> VeinTemplate {
    let pixels: Vec<u8> = (0..(w as usize * h as usize))
        .map(|i| {
            let x = i % w as usize;
            let y = i / w as usize;
            if (x + y) % 2 == 0 {
                255
            } else {
                0
            }
        })
        .collect();
    VeinTemplate {
        template_id: "checker".to_string(),
        vein_type: VeinType::FingerVein,
        body_side: BodySide::R,
        nir_wavelength_nm: 850,
        width: w,
        height: h,
        pattern_bitmap: pixels,
        captured_at: Utc::now(),
    }
}

// Test 1: parse round-trip
#[test]
fn test_parse_round_trip() {
    let tmpl = make_template(8, 8, 128, VeinType::PalmVein, BodySide::L);
    let bytes = serialize_nir_bitmap(&tmpl);
    let parsed = parse_nir_bitmap(&bytes).expect("should parse");
    assert_eq!(parsed.width, tmpl.width);
    assert_eq!(parsed.height, tmpl.height);
    assert_eq!(parsed.nir_wavelength_nm, tmpl.nir_wavelength_nm);
    assert_eq!(parsed.vein_type, tmpl.vein_type);
    assert_eq!(parsed.body_side, tmpl.body_side);
    assert_eq!(parsed.pattern_bitmap, tmpl.pattern_bitmap);
}

// Test 2: parse rejects wrong magic
#[test]
fn test_parse_wrong_magic() {
    let mut bytes =
        serialize_nir_bitmap(&make_template(4, 4, 0, VeinType::FingerVein, BodySide::R));
    bytes[0] = 0xFF;
    assert!(matches!(
        parse_nir_bitmap(&bytes),
        Err(VeinError::InvalidMagic)
    ));
}

// Test 3: parse rejects truncated pixel data
#[test]
fn test_parse_truncated_pixel_data() {
    let tmpl = make_template(8, 8, 200, VeinType::WristVein, BodySide::L);
    let mut bytes = serialize_nir_bitmap(&tmpl);
    // Remove last 10 bytes of pixel data.
    let len = bytes.len();
    bytes.truncate(len - 10);
    assert!(matches!(
        parse_nir_bitmap(&bytes),
        Err(VeinError::PixelDataMismatch { .. })
    ));
}

// Test 4: unknown VeinType byte returns error
#[test]
fn test_parse_unknown_vein_type() {
    let tmpl = make_template(4, 4, 0, VeinType::FingerVein, BodySide::R);
    let mut bytes = serialize_nir_bitmap(&tmpl);
    // VeinType byte is at offset 10 (4 magic + 2 width + 2 height + 2 wavelength).
    bytes[10] = 99;
    assert!(matches!(
        parse_nir_bitmap(&bytes),
        Err(VeinError::UnknownVeinType(99))
    ));
}

// Test 5: ncc of identical bitmaps returns 1.0
#[test]
fn test_ncc_identical() {
    let a: Vec<u8> = (0u8..64).collect();
    let score = ncc(&a, &a);
    assert!((score - 1.0).abs() < 1e-4, "expected 1.0, got {score}");
}

// Test 6: ncc of inverted bitmaps returns -1.0
#[test]
fn test_ncc_inverted() {
    let a: Vec<u8> = (0u8..64).collect();
    let b: Vec<u8> = a.iter().map(|&v| 255 - v).collect();
    let score = ncc(&a, &b);
    assert!((score + 1.0).abs() < 1e-4, "expected -1.0, got {score}");
}

// Test 7: match_vein genuine pair returns score > DEFAULT_THRESHOLD
#[test]
fn test_match_genuine_pair() {
    let tmpl = checkerboard(32, 32);
    let score = match_vein(&tmpl, &tmpl);
    assert!(
        score.raw >= DEFAULT_THRESHOLD,
        "genuine pair score {} should exceed threshold {}",
        score.raw,
        DEFAULT_THRESHOLD
    );
    assert!(score.threshold_pass);
}

// Test 8: match_vein imposter pair returns score < DEFAULT_THRESHOLD
#[test]
fn test_match_imposter_pair() {
    let probe = make_template(32, 32, 200, VeinType::FingerVein, BodySide::R);
    // Create a noisy "random" gallery by alternating values.
    let gallery_pixels: Vec<u8> = (0..1024u32).map(|i| ((i * 73 + 17) % 256) as u8).collect();
    let gallery = VeinTemplate {
        template_id: "gallery".to_string(),
        vein_type: VeinType::FingerVein,
        body_side: BodySide::R,
        nir_wavelength_nm: 850,
        width: 32,
        height: 32,
        pattern_bitmap: gallery_pixels,
        captured_at: Utc::now(),
    };
    let score = match_vein(&probe, &gallery);
    assert!(
        score.raw < DEFAULT_THRESHOLD,
        "imposter score {} should be below threshold {}",
        score.raw,
        DEFAULT_THRESHOLD
    );
}

// Test 9: quality contrast of uniform image = 0.0
#[test]
fn test_quality_uniform_contrast_zero() {
    let tmpl = make_template(16, 16, 128, VeinType::FingerVein, BodySide::R);
    let q = score_quality(&tmpl);
    assert_eq!(q.contrast, 0.0);
}

// Test 10: quality contrast of binary image > 0.5
#[test]
fn test_quality_binary_contrast() {
    let tmpl = checkerboard(16, 16);
    let q = score_quality(&tmpl);
    assert!(
        q.contrast > 0.5,
        "binary image contrast should exceed 0.5, got {}",
        q.contrast
    );
}

// Test 11: branch density of empty bitmap = 0.0
#[test]
fn test_branch_density_empty() {
    let tmpl = make_template(16, 16, 0, VeinType::FingerVein, BodySide::R);
    let q = score_quality(&tmpl);
    assert_eq!(q.branch_density, 0.0);
}

// Test 12: quality overall in [0, 100]
#[test]
fn test_quality_overall_in_range() {
    let tmpl = checkerboard(16, 16);
    let q = score_quality(&tmpl);
    assert!(
        q.overall >= 0.0 && q.overall <= 100.0,
        "overall {} out of range",
        q.overall
    );
}

// Test 13: pixel data length mismatch returns error
#[test]
fn test_parse_pixel_data_extra() {
    let tmpl = make_template(4, 4, 50, VeinType::FingerVein, BodySide::R);
    let mut bytes = serialize_nir_bitmap(&tmpl);
    // Add extra bytes so pixel section mismatches.
    bytes.push(0xFF);
    assert!(matches!(
        parse_nir_bitmap(&bytes),
        Err(VeinError::PixelDataMismatch { .. })
    ));
}

// Test 14: rotation search finds correct rotation offset
#[test]
fn test_rotation_search() {
    // The score returned for a genuine pair should indicate the best rotation was found.
    let tmpl = checkerboard(32, 32);
    let score = match_vein(&tmpl, &tmpl);
    // At 0 degree rotation a genuine pair should match at 0 deg.
    assert_eq!(score.best_rotation_deg, 0.0);
}
