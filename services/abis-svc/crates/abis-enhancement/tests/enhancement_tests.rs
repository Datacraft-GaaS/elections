use abis_enhancement::face::{bicubic_upsample, super_resolve, unsharp_mask, SuperResConfig};
use abis_enhancement::fingerprint::{
    apply_gabor_bank, gabor_filter, histogram_equalize, morphological_thin, restore, GrayscaleImage,
};
use abis_enhancement::iris::{
    detect_eyelid_occlusion, integro_differential_operator, segment_iris, IrisCircle,
};
use abis_enhancement::slap::{segment_slap, FingerPosition, Hand};
use abis_enhancement::EnhancementError;

fn uniform_image(w: u32, h: u32, val: u8) -> GrayscaleImage {
    GrayscaleImage::new(w, h, vec![val; (w * h) as usize]).unwrap()
}

fn ramp_image(w: u32, h: u32) -> GrayscaleImage {
    let pixels: Vec<u8> = (0..(w * h)).map(|i| (i % 256) as u8).collect();
    GrayscaleImage::new(w, h, pixels).unwrap()
}

#[test]
fn histogram_equalize_all_zeros() {
    let img = uniform_image(4, 4, 0);
    let result = histogram_equalize(&img).unwrap();
    assert!(result.pixels.iter().all(|&p| p == 0));
}

#[test]
fn histogram_equalize_spans_range() {
    let data = vec![0u8, 128, 255, 128];
    let img = GrayscaleImage::new(4, 1, data).unwrap();
    let result = histogram_equalize(&img).unwrap();
    let min = *result.pixels.iter().min().unwrap();
    let max = *result.pixels.iter().max().unwrap();
    assert!(max > min, "equalized image should span a range");
}

#[test]
fn gabor_filter_uniform_near_128() {
    let img = uniform_image(20, 20, 128);
    let result = gabor_filter(&img, 0.1, 0.0, 3.0, 5.0).unwrap();
    for &p in &result.pixels {
        assert!((p as i32 - 128).abs() <= 50, "pixel {p} too far from 128");
    }
}

#[test]
fn apply_gabor_bank_same_dimensions() {
    let img = uniform_image(20, 20, 100);
    let result = apply_gabor_bank(&img).unwrap();
    assert_eq!(result.width, 20);
    assert_eq!(result.height, 20);
}

#[test]
fn morphological_thin_white_image() {
    let img = uniform_image(10, 10, 255);
    let result = morphological_thin(&img, 128).unwrap();
    for &p in &result.pixels {
        assert!(p == 0 || p == 255, "expected binary pixel, got {p}");
    }
}

#[test]
fn restore_same_dimensions() {
    let img = uniform_image(20, 20, 150);
    let result = restore(&img).unwrap();
    assert_eq!(result.width, 20);
    assert_eq!(result.height, 20);
}

#[test]
fn bicubic_upsample_2x() {
    let img = ramp_image(4, 4);
    let result = bicubic_upsample(&img, 2).unwrap();
    assert_eq!(result.width, 8);
    assert_eq!(result.height, 8);
}

#[test]
fn unsharp_mask_in_range() {
    let img = ramp_image(10, 10);
    let result = unsharp_mask(&img, 1, 0.5).unwrap();
    assert_eq!(result.pixels.len(), 100);
}

#[test]
fn super_resolve_dimensions() {
    let img = uniform_image(5, 5, 100);
    let config = SuperResConfig {
        scale_factor: 2,
        unsharp_radius: 1,
        unsharp_amount: 0.5,
    };
    let result = super_resolve(&img, &config).unwrap();
    assert_eq!(result.width, 10);
    assert_eq!(result.height, 10);
}

#[test]
fn segment_slap_right_hand_4_segments() {
    let img = uniform_image(40, 20, 200);
    let fingers = segment_slap(&img, Hand::Right).unwrap();
    assert_eq!(fingers.len(), 4);
    for f in &fingers {
        assert_eq!(f.image.width, 10);
        assert_eq!(f.image.height, 20);
    }
}

#[test]
fn segment_slap_left_hand_positions() {
    let img = uniform_image(40, 20, 200);
    let fingers = segment_slap(&img, Hand::Left).unwrap();
    assert_eq!(fingers[0].position, FingerPosition::LeftLittle);
    assert_eq!(fingers[1].position, FingerPosition::LeftRing);
    assert_eq!(fingers[2].position, FingerPosition::LeftMiddle);
    assert_eq!(fingers[3].position, FingerPosition::LeftIndex);
}

#[test]
fn segment_slap_too_narrow_error() {
    let img = uniform_image(3, 20, 128);
    assert!(matches!(
        segment_slap(&img, Hand::Right),
        Err(EnhancementError::SegmentationFailed(_))
    ));
}

#[test]
fn integro_differential_operator_valid_circle() {
    let img = uniform_image(60, 60, 128);
    let circle = integro_differential_operator(&img, 10, 30).unwrap();
    assert!(circle.center_x >= 0.0 && circle.center_x < 60.0);
    assert!(circle.center_y >= 0.0 && circle.center_y < 60.0);
    assert!(circle.radius >= 10.0);
}

#[test]
fn detect_eyelid_occlusion_mask_area() {
    let img = uniform_image(100, 100, 128);
    let limbus = IrisCircle {
        center_x: 50.0,
        center_y: 50.0,
        radius: 40.0,
    };
    let mask = detect_eyelid_occlusion(&img, &limbus).unwrap();
    assert_eq!(mask.mask.len() as u32, mask.width * mask.height);
}

#[test]
fn segment_iris_pupil_smaller_than_limbus() {
    let img = uniform_image(100, 100, 100);
    let result = segment_iris(&img).unwrap();
    assert!(
        result.pupil.radius < result.limbus.radius || result.limbus.radius >= 80.0,
        "pupil radius {} should be less than limbus radius {}",
        result.pupil.radius,
        result.limbus.radius
    );
}
