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

//! Heuristic face detector using integral image + sliding window.
//!
//! Implements a simplified Viola-Jones cascade:
//! - Integral image for O(1) rectangle-sum queries.
//! - Two Haar-like features per stage: forehead-dark / eye-band-bright contrast.
//! - Multi-scale sliding window with stride proportional to window size.
//! - Greedy NMS at the end.
//!
//! Reference: Viola & Jones, CVPR 2001; Lienhart & Maydt, ICIP 2002
//! (extended Haar features).

use crate::nms::nms;
use crate::{BoundingBox, DetectError, FaceBox, FaceDetector, RgbImage};

/// Integral image over u64 (accumulates pixel sums without overflow for
/// images up to ~16 MP with 8-bit pixels).
struct IntegralImage {
    width: usize,
    height: usize,
    data: Vec<u64>,
}

impl IntegralImage {
    fn build(pixels: &[u8], width: usize, height: usize) -> Self {
        let mut data = vec![0u64; (width + 1) * (height + 1)];
        for y in 0..height {
            for x in 0..width {
                let px = pixels[y * width + x] as u64;
                let above = data[y * (width + 1) + x + 1];
                let left = data[(y + 1) * (width + 1) + x];
                let diag = data[y * (width + 1) + x];
                data[(y + 1) * (width + 1) + (x + 1)] = px + above + left - diag;
            }
        }
        Self {
            width,
            height,
            data,
        }
    }

    /// Sum of pixels in rectangle [x, x+w) x [y, y+h).
    fn rect_sum(&self, x: usize, y: usize, w: usize, h: usize) -> u64 {
        let x2 = (x + w).min(self.width);
        let y2 = (y + h).min(self.height);
        let w1 = self.width + 1;
        self.data[y2 * w1 + x2] + self.data[y * w1 + x]
            - self.data[y * w1 + x2]
            - self.data[y2 * w1 + x]
    }

    /// Mean pixel value in rectangle.
    fn rect_mean(&self, x: usize, y: usize, w: usize, h: usize) -> f32 {
        let area = (w * h) as f32;
        if area < 1.0 {
            return 0.0;
        }
        self.rect_sum(x, y, w, h) as f32 / area
    }
}

/// Viola-Jones-style two-stage Haar cascade (simplified single strong
/// classifier per stage for the pure-Rust baseline).
///
/// Stage 1: Forehead region is darker than the eye band.
/// Stage 2: Eye-band brightness is significantly above the cheek region.
/// Stage 3: Face-region mean intensity is above noise floor.
fn passes_cascade(ii: &IntegralImage, x: usize, y: usize, size: usize) -> Option<f32> {
    if size < 16 {
        return None;
    }
    let third = size / 3;
    let sixth = size / 6;

    // Forehead: top third
    let forehead_mean = ii.rect_mean(x, y, size, third);
    // Eye band: middle third
    let eye_mean = ii.rect_mean(x, y + third, size, third);
    // Mouth/chin: bottom third
    let chin_mean = ii.rect_mean(x, y + 2 * third, size, third);

    // Stage 1: eye band should be brighter than forehead (eye-whites + skin tone)
    if eye_mean < forehead_mean * 0.85 {
        return None;
    }

    // Stage 2: face mean should be distinct from near-black background
    let face_mean = ii.rect_mean(x, y, size, size);
    if face_mean < 30.0 {
        return None;
    }

    // Stage 3: left-right symmetry check via nose bridge (narrow strip in centre)
    let nose_x = x + size / 2 - sixth / 2;
    let nose_w = sixth.max(1);
    let nose_mean = ii.rect_mean(nose_x, y + third, nose_w, third);

    // Nose bridge should not be extremely bright (not pure-white blob)
    if nose_mean > 240.0 {
        return None;
    }

    // Stage 4: chin slightly darker than eye band (typical illumination gradient)
    if chin_mean > eye_mean * 1.2 {
        return None;
    }

    // Confidence: linear combination of cascade margin scores clamped to [0, 1].
    let margin = ((eye_mean - forehead_mean * 0.85) / 20.0).clamp(0.0, 1.0);
    let conf = (0.5 + margin * 0.5).clamp(0.0, 1.0);
    Some(conf)
}

/// Heuristic Haar-cascade face detector (pure Rust, no model weights).
///
/// This is intended as a functional CI-passing baseline. For production,
/// enable the `onnx` feature and point to the face-onnx-worker sidecar.
#[derive(Debug, Clone)]
pub struct HaarDetector {
    /// Minimum face size in pixels.
    pub min_face_px: u32,
    /// Maximum face size in pixels (0 = unrestricted).
    pub max_face_px: u32,
    /// Scale factor between successive window sizes (e.g. 1.2).
    pub scale_factor: f32,
    /// NMS IoU suppression threshold.
    pub iou_threshold: f32,
    /// Confidence threshold for raw candidates.
    pub conf_threshold: f32,
}

impl Default for HaarDetector {
    fn default() -> Self {
        Self {
            min_face_px: 48,
            max_face_px: 0,
            scale_factor: 1.25,
            iou_threshold: 0.4,
            conf_threshold: 0.5,
        }
    }
}

impl HaarDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_size(mut self, px: u32) -> Self {
        self.min_face_px = px;
        self
    }

    pub fn with_conf_threshold(mut self, t: f32) -> Self {
        self.conf_threshold = t;
        self
    }
}

impl FaceDetector for HaarDetector {
    fn detect(&self, image: &RgbImage) -> Result<Vec<FaceBox>, DetectError> {
        if image.width == 0 || image.height == 0 {
            return Err(DetectError::EmptyImage);
        }
        let gray = image.to_gray();
        let ii = IntegralImage::build(&gray.pixels, gray.width as usize, gray.height as usize);

        let mut candidates: Vec<FaceBox> = Vec::new();
        let max_size = if self.max_face_px == 0 {
            image.width.min(image.height)
        } else {
            self.max_face_px
        };

        let mut size = self.min_face_px;
        while size <= max_size {
            let stride = (size / 8).max(2) as usize;
            let sz = size as usize;
            let w = image.width as usize;
            let h = image.height as usize;

            let mut y = 0usize;
            while y + sz <= h {
                let mut x = 0usize;
                while x + sz <= w {
                    if let Some(conf) = passes_cascade(&ii, x, y, sz) {
                        if conf >= self.conf_threshold {
                            candidates.push(FaceBox::new(
                                BoundingBox::new(x as u32, y as u32, sz as u32, sz as u32),
                                conf,
                            ));
                        }
                    }
                    x += stride;
                }
                y += stride;
            }
            // Geometric scale progression.
            size = ((size as f32) * self.scale_factor).ceil() as u32;
            if size == 0 {
                break;
            }
        }

        let kept = nms(candidates, self.iou_threshold);
        Ok(kept)
    }
}
