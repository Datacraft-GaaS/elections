//! Tests for the texture-based fingerprint matching crate.

use eel_abis_fp_texture::{
    fingercode::{
        encode_fingercode, filter_bank, locate_reference_point, match_fingercode, tessellate,
    },
    lbp::{
        chi_square_distance, chi_square_similarity, lbp_histogram, lbp_histogram_for_image,
        LBP_BINS,
    },
    matcher::{TextureMatcher, TextureWeights},
    mb_lbp::{mb_lbp_histogram, mb_lbp_multi_scale, MbLbpBlockSize},
    GrayImage, Point,
};

// Helper: create a flat gray image.
fn flat_image(width: u32, height: u32, value: u8) -> GrayImage {
    GrayImage::new(width, height, vec![value; (width * height) as usize]).unwrap()
}

// Helper: create a striped image (vertical ridges).
fn striped_image(width: u32, height: u32, period: u32) -> GrayImage {
    let pixels: Vec<u8> = (0..(width * height))
        .map(|i| {
            let x = i % width;
            if (x / period) % 2 == 0 {
                50
            } else {
                200
            }
        })
        .collect();
    GrayImage::new(width, height, pixels).unwrap()
}

// Helper: create a horizontally striped image.
fn h_striped_image(width: u32, height: u32, period: u32) -> GrayImage {
    let pixels: Vec<u8> = (0..(width * height))
        .map(|i| {
            let y = i / width;
            if (y / period) % 2 == 0 {
                50
            } else {
                200
            }
        })
        .collect();
    GrayImage::new(width, height, pixels).unwrap()
}

// ----- GrayImage construction -----

#[test]
fn test_gray_image_correct_size() {
    let img = GrayImage::new(4, 4, vec![0u8; 16]);
    assert!(img.is_ok());
}

#[test]
fn test_gray_image_wrong_size_errors() {
    let img = GrayImage::new(4, 4, vec![0u8; 10]);
    assert!(img.is_err());
}

#[test]
fn test_gray_image_pixel_access() {
    let mut img = flat_image(10, 10, 100);
    img.set_pixel(5, 3, 42);
    assert_eq!(img.pixel(5, 3), 42);
}

// ----- Reference point -----

#[test]
fn test_reference_point_returns_for_large_image() {
    let img = striped_image(64, 64, 5);
    let rp = locate_reference_point(&img);
    assert!(
        rp.is_ok(),
        "should return a reference point on a 64x64 image"
    );
    let p = rp.unwrap();
    assert!(p.x > 0.0 && p.x < 64.0);
    assert!(p.y > 0.0 && p.y < 64.0);
}

#[test]
fn test_reference_point_too_small_errors() {
    let img = flat_image(8, 8, 128);
    assert!(locate_reference_point(&img).is_err());
}

#[test]
fn test_reference_point_center_for_flat_image() {
    // Flat image has no gradient, falls back to centre.
    let img = flat_image(64, 64, 128);
    let p = locate_reference_point(&img).unwrap();
    assert!((p.x - 32.0).abs() < 16.0);
    assert!((p.y - 32.0).abs() < 16.0);
}

// ----- Tessellation -----

#[test]
fn test_tessellate_produces_80_sectors() {
    let img = flat_image(300, 300, 128);
    let rp = Point::new(150.0, 150.0);
    let sectors = tessellate(&img, rp);
    assert_eq!(sectors.len(), 80);
}

#[test]
fn test_tessellate_all_pixels_assigned_are_in_bounds() {
    let img = striped_image(200, 200, 8);
    let rp = Point::new(100.0, 100.0);
    let sectors = tessellate(&img, rp);
    for s in &sectors {
        for &(x, y) in &s.indices {
            assert!(x < 200, "x={x} out of bounds");
            assert!(y < 200, "y={y} out of bounds");
        }
    }
}

// ----- Filter bank -----

#[test]
fn test_filter_bank_produces_8_images() {
    let img = striped_image(64, 64, 8);
    let fb = filter_bank(&img, &[]);
    assert_eq!(fb.len(), 8);
}

#[test]
fn test_filter_bank_output_same_dimensions() {
    let img = striped_image(64, 64, 8);
    let fb = filter_bank(&img, &[]);
    for f in &fb {
        assert_eq!(f.width, 64);
        assert_eq!(f.height, 64);
    }
}

#[test]
fn test_filter_bank_non_negative() {
    let img = striped_image(64, 64, 8);
    let fb = filter_bank(&img, &[]);
    for f in &fb {
        assert!(
            f.pixels.iter().all(|&v| v >= 0.0),
            "filter output should be non-negative"
        );
    }
}

// ----- FingerCode encoding and matching -----

#[test]
fn test_encode_fingercode_length() {
    let img = striped_image(200, 200, 8);
    let rp = Point::new(100.0, 100.0);
    let sectors = tessellate(&img, rp);
    let filtered = filter_bank(&img, &[]);
    let fc = encode_fingercode(&sectors, &filtered);
    assert_eq!(fc.features.len(), 640);
}

#[test]
fn test_match_identical_fingercode_returns_one() {
    let img = striped_image(200, 200, 8);
    let rp = Point::new(100.0, 100.0);
    let sectors = tessellate(&img, rp);
    let filtered = filter_bank(&img, &[]);
    let fc = encode_fingercode(&sectors, &filtered);
    let score = match_fingercode(&fc, &fc);
    assert!(
        (score - 1.0).abs() < 1e-5,
        "identical FingerCodes must score 1.0, got {score}"
    );
}

#[test]
fn test_match_different_patterns_below_threshold() {
    // Vertical stripes vs flat -- very different textures.
    let img_a = striped_image(200, 200, 8);
    let img_b = flat_image(200, 200, 200);
    let rp = Point::new(100.0, 100.0);
    let sectors_a = tessellate(&img_a, rp);
    let sectors_b = tessellate(&img_b, rp);
    let fa = filter_bank(&img_a, &[]);
    let fb_filtered = filter_bank(&img_b, &[]);
    let fc_a = encode_fingercode(&sectors_a, &fa);
    let fc_b = encode_fingercode(&sectors_b, &fb_filtered);
    let score = match_fingercode(&fc_a, &fc_b);
    assert!(
        score < 0.80,
        "different patterns should score below 0.80, got {score}"
    );
}

// ----- LBP -----

#[test]
fn test_lbp_histogram_length_is_256() {
    let img = striped_image(64, 64, 5);
    let h = lbp_histogram_for_image(&img);
    assert_eq!(h.len(), LBP_BINS);
}

#[test]
fn test_lbp_histogram_sums_to_one() {
    let img = striped_image(64, 64, 5);
    let h = lbp_histogram_for_image(&img);
    let sum: f32 = h.iter().sum();
    assert!(
        (sum - 1.0).abs() < 1e-5,
        "normalized histogram must sum to 1.0, got {sum}"
    );
}

#[test]
fn test_lbp_histogram_from_codes_length() {
    let codes: Vec<u8> = (0..256).map(|i| i as u8).collect();
    let h = lbp_histogram(&codes);
    assert_eq!(h.len(), LBP_BINS);
}

#[test]
fn test_lbp_identical_images_zero_distance() {
    let img = striped_image(64, 64, 5);
    let h = lbp_histogram_for_image(&img);
    let d = chi_square_distance(&h, &h);
    assert!(
        d < 1e-6,
        "identical histogram must have zero chi-square distance, got {d}"
    );
}

#[test]
fn test_lbp_different_images_nonzero_distance() {
    let a = striped_image(64, 64, 5);
    let b = h_striped_image(64, 64, 5);
    let ha = lbp_histogram_for_image(&a);
    let hb = lbp_histogram_for_image(&b);
    let d = chi_square_distance(&ha, &hb);
    assert!(
        d > 0.0,
        "different textures should have nonzero chi-square distance"
    );
}

#[test]
fn test_lbp_histogram_length_independent_of_image_size() {
    let small = striped_image(32, 32, 4);
    let large = striped_image(128, 128, 4);
    let hs = lbp_histogram_for_image(&small);
    let hl = lbp_histogram_for_image(&large);
    assert_eq!(
        hs.len(),
        hl.len(),
        "LBP histogram length must not depend on image size"
    );
}

// ----- MB-LBP -----

#[test]
fn test_mb_lbp_histogram_length_256() {
    let img = striped_image(64, 64, 5);
    let h = mb_lbp_histogram(&img, MbLbpBlockSize::S3);
    assert_eq!(h.len(), 256);
}

#[test]
fn test_mb_lbp_multi_scale_length_768() {
    let img = striped_image(64, 64, 5);
    let h = mb_lbp_multi_scale(&img);
    assert_eq!(h.len(), 768);
}

#[test]
fn test_mb_lbp_identical_similarity_near_one() {
    let img = striped_image(64, 64, 5);
    let h = mb_lbp_multi_scale(&img);
    let sim = crate::mb_lbp::mb_lbp_similarity(&h, &h);
    assert!(
        sim > 0.99,
        "identical MB-LBP features must have similarity near 1.0, got {sim}"
    );
}

// ----- TextureMatcher -----

#[test]
fn test_texture_matcher_identical_images_high_score() {
    let img = striped_image(200, 200, 8);
    let m = TextureMatcher::default();
    let result = m.compare(&img, &img).unwrap();
    assert!(
        result.fused > 0.8,
        "identical images should score > 0.8, got {}",
        result.fused
    );
}

#[test]
fn test_texture_matcher_different_images_lower_score() {
    let a = striped_image(200, 200, 8);
    let b = flat_image(200, 200, 200);
    let m = TextureMatcher::default();
    let sa = m.compare(&a, &a).unwrap().fused;
    let sab = m.compare(&a, &b).unwrap().fused;
    assert!(sa > sab, "self-match should score higher than cross-match");
}

// re-export for mb_lbp_similarity use in test above
mod mb_lbp {
    pub use eel_abis_fp_texture::mb_lbp::mb_lbp_similarity;
}
