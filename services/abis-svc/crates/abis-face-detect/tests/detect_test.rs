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

use eel_abis_face_detect::{
    nms, BoundingBox, FaceBox, FaceDetector, GrayImage, HaarDetector, RgbImage,
};

// Helpers

fn dark_rgb(w: u32, h: u32) -> RgbImage {
    RgbImage::new(w, h, vec![10u8; (w * h * 3) as usize]).unwrap()
}

/// Build an RGB image that has a bright oval region simulating a face.
fn synthetic_face_image(w: u32, h: u32, fx: u32, fy: u32, fw: u32, fh: u32) -> RgbImage {
    let mut pixels = vec![15u8; (w * h * 3) as usize]; // dark background
    for py in fy..fy + fh {
        for px in fx..fx + fw {
            let idx = ((py * w + px) * 3) as usize;
            if idx + 2 < pixels.len() {
                // Bright skin-tone: upper 1/3 darker (forehead), middle 1/3 bright (eyes)
                let rel_y = py - fy;
                let v: u8 = if rel_y < fh / 3 {
                    80
                } else if rel_y < 2 * fh / 3 {
                    160
                } else {
                    100
                };
                pixels[idx] = v;
                pixels[idx + 1] = v;
                pixels[idx + 2] = v;
            }
        }
    }
    RgbImage::new(w, h, pixels).unwrap()
}

fn fb(x: u32, y: u32, w: u32, h: u32, conf: f32) -> FaceBox {
    FaceBox::new(BoundingBox::new(x, y, w, h), conf)
}

// --- BoundingBox tests ---

#[test]
fn bbox_iou_identical_boxes() {
    let a = BoundingBox::new(0, 0, 10, 10);
    assert!((a.iou(&a) - 1.0).abs() < 1e-5);
}

#[test]
fn bbox_iou_non_overlapping() {
    let a = BoundingBox::new(0, 0, 10, 10);
    let b = BoundingBox::new(20, 20, 10, 10);
    assert!(a.iou(&b) < 1e-5);
}

#[test]
fn bbox_iou_partial_overlap() {
    let a = BoundingBox::new(0, 0, 10, 10);
    let b = BoundingBox::new(5, 0, 10, 10);
    let iou = a.iou(&b);
    assert!(iou > 0.0 && iou < 1.0);
}

#[test]
fn bbox_area_correct() {
    let b = BoundingBox::new(0, 0, 7, 3);
    assert_eq!(b.area(), 21);
}

#[test]
fn bbox_centre_correct() {
    let b = BoundingBox::new(0, 0, 10, 10);
    let (cx, cy) = b.centre();
    assert!((cx - 5.0).abs() < 1e-5 && (cy - 5.0).abs() < 1e-5);
}

// --- GrayImage / RgbImage tests ---

#[test]
fn gray_image_rejects_zero_dimensions() {
    assert!(GrayImage::new(0, 10, vec![]).is_err());
    assert!(GrayImage::new(10, 0, vec![]).is_err());
}

#[test]
fn rgb_image_rejects_zero_dimensions() {
    assert!(RgbImage::new(0, 5, vec![]).is_err());
}

#[test]
fn rgb_to_gray_luminance() {
    // Pure red pixel
    let img = RgbImage::new(1, 1, vec![255, 0, 0]).unwrap();
    let g = img.to_gray();
    // BT.601: 0.299 * 255 ~ 76
    assert!((g.pixels[0] as i32 - 76).abs() <= 2);
}

#[test]
fn rgb_to_gray_white_pixel() {
    let img = RgbImage::new(1, 1, vec![255, 255, 255]).unwrap();
    let g = img.to_gray();
    assert_eq!(g.pixels[0], 255);
}

// --- NMS tests ---

#[test]
fn nms_collapses_overlapping_boxes() {
    let boxes = vec![
        fb(0, 0, 10, 10, 0.95),
        fb(1, 1, 10, 10, 0.80),
        fb(2, 2, 10, 10, 0.70),
    ];
    let result = nms(boxes, 0.4);
    assert_eq!(result.len(), 1);
    assert!((result[0].confidence - 0.95).abs() < 1e-5);
}

#[test]
fn nms_preserves_distinct_boxes() {
    let boxes = vec![
        fb(0, 0, 20, 20, 0.9),
        fb(100, 0, 20, 20, 0.8),
        fb(200, 0, 20, 20, 0.7),
    ];
    let result = nms(boxes, 0.5);
    assert_eq!(result.len(), 3);
}

#[test]
fn nms_empty_returns_empty() {
    let result = nms(vec![], 0.5);
    assert!(result.is_empty());
}

#[test]
fn nms_output_sorted_descending_confidence() {
    let boxes = vec![
        fb(0, 0, 10, 10, 0.6),
        fb(50, 50, 10, 10, 0.9),
        fb(100, 100, 10, 10, 0.75),
    ];
    let result = nms(boxes, 0.5);
    for i in 0..result.len().saturating_sub(1) {
        assert!(result[i].confidence >= result[i + 1].confidence);
    }
}

// --- HaarDetector tests ---

#[test]
fn detector_empty_image_error() {
    // We cannot build a zero-size RgbImage, but we can test that the detector
    // itself returns Ok([]) on an all-dark image with no face-like regions.
    let det = HaarDetector::new();
    let dark = dark_rgb(64, 64);
    let result = det.detect(&dark).unwrap();
    // Dark uniform image: no face-like contrast patterns expected.
    // Allow zero or a small number of false positives.
    assert!(
        result.len() <= 2,
        "too many false positives on dark: {}",
        result.len()
    );
}

#[test]
fn detector_detect_gray_wrapper_compiles() {
    let det = HaarDetector::new();
    let gray = GrayImage::new(32, 32, vec![128u8; 32 * 32]).unwrap();
    let _ = det.detect_gray(&gray).unwrap();
}

#[test]
fn detector_default_parameters() {
    let det = HaarDetector::default();
    assert_eq!(det.min_face_px, 48);
    assert!((det.scale_factor - 1.25).abs() < 1e-5);
}

#[test]
fn detector_builder_methods() {
    let det = HaarDetector::new()
        .with_min_size(32)
        .with_conf_threshold(0.3);
    assert_eq!(det.min_face_px, 32);
    assert!((det.conf_threshold - 0.3).abs() < 1e-5);
}

#[test]
fn detector_synthetic_face_returns_candidates() {
    // Build a 200x200 image with a synthetic face-like region at (50, 50, 100x100).
    let det = HaarDetector::new()
        .with_min_size(48)
        .with_conf_threshold(0.45);
    let img = synthetic_face_image(200, 200, 50, 50, 100, 100);
    let result = det.detect(&img).unwrap();
    // Should detect at least one candidate in the face region.
    assert!(
        !result.is_empty(),
        "expected at least one detection on synthetic face"
    );
}
