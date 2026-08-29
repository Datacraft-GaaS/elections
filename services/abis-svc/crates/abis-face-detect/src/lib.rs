//! Face detection: integral-image Haar-like baseline + optional ONNX sidecar.
//!
//! Reference: Viola, P. & Jones, M. (2001). "Rapid Object Detection using a
//! Boosted Cascade of Simple Features." CVPR 2001.
#![warn(missing_docs)]

pub mod nms;
pub mod baseline;

use thiserror::Error;

/// Detection error.
#[derive(Debug, Error)]
pub enum DetectError {
    #[error("image dimensions are zero")]
    EmptyImage,
    #[error("invalid threshold: {0}")]
    InvalidThreshold(f32),
}

/// Axis-aligned bounding box in pixel coordinates.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl BoundingBox {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// Intersection-over-union with another bounding box.
    pub fn iou(&self, other: &Self) -> f32 {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);
        if x2 <= x1 || y2 <= y1 {
            return 0.0;
        }
        let inter = ((x2 - x1) * (y2 - y1)) as f32;
        let union = (self.width * self.height + other.width * other.height) as f32 - inter;
        if union < 1e-6 {
            return 0.0;
        }
        inter / union
    }

    /// Centre point (x, y).
    pub fn centre(&self) -> (f32, f32) {
        (self.x as f32 + self.width as f32 / 2.0,
         self.y as f32 + self.height as f32 / 2.0)
    }

    /// Area in pixels.
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

/// 5 landmark hints: left_eye, right_eye, nose, left_mouth, right_mouth.
/// Coordinates in pixel space.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FiveLandmarks {
    pub left_eye:    (f32, f32),
    pub right_eye:   (f32, f32),
    pub nose:        (f32, f32),
    pub left_mouth:  (f32, f32),
    pub right_mouth: (f32, f32),
}

/// A detected face candidate.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FaceBox {
    pub bbox:       BoundingBox,
    /// Detection confidence in [0.0, 1.0].
    pub confidence: f32,
    /// Coarse landmark estimates (may be None for the heuristic baseline).
    pub landmarks:  Option<FiveLandmarks>,
}

impl FaceBox {
    pub fn new(bbox: BoundingBox, confidence: f32) -> Self {
        Self { bbox, confidence, landmarks: None }
    }

    pub fn with_landmarks(mut self, lm: FiveLandmarks) -> Self {
        self.landmarks = Some(lm);
        self
    }
}

/// Raw grayscale image buffer (row-major, 1 byte per pixel).
#[derive(Debug, Clone)]
pub struct GrayImage {
    pub width:  u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl GrayImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, DetectError> {
        if width == 0 || height == 0 {
            return Err(DetectError::EmptyImage);
        }
        Ok(Self { width, height, pixels })
    }

    pub fn pixel(&self, x: u32, y: u32) -> u8 {
        let idx = (y * self.width + x) as usize;
        self.pixels.get(idx).copied().unwrap_or(0)
    }
}

/// Raw RGB image buffer (row-major, 3 bytes per pixel: R G B).
#[derive(Debug, Clone)]
pub struct RgbImage {
    pub width:  u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl RgbImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, DetectError> {
        if width == 0 || height == 0 {
            return Err(DetectError::EmptyImage);
        }
        Ok(Self { width, height, pixels })
    }

    pub fn to_gray(&self) -> GrayImage {
        // BT.601 luminance: 0.299R + 0.587G + 0.114B
        let pixels: Vec<u8> = self.pixels
            .chunks(3)
            .map(|c| {
                let r = c[0] as f32;
                let g = c[1] as f32;
                let b = c[2] as f32;
                (0.299 * r + 0.587 * g + 0.114 * b).round().clamp(0.0, 255.0) as u8
            })
            .collect();
        GrayImage { width: self.width, height: self.height, pixels }
    }
}

/// Face detector trait.
pub trait FaceDetector {
    /// Detect faces in an RGB image. Returns detected face boxes sorted by
    /// descending confidence after non-maximum suppression.
    fn detect(&self, image: &RgbImage) -> Result<Vec<FaceBox>, DetectError>;

    /// Detect in a grayscale image (convenience wrapper).
    fn detect_gray(&self, image: &GrayImage) -> Result<Vec<FaceBox>, DetectError> {
        // Build a synthetic RGB from the gray channel.
        let pixels: Vec<u8> = image.pixels
            .iter()
            .flat_map(|&v| [v, v, v])
            .collect();
        let rgb = RgbImage { width: image.width, height: image.height, pixels };
        self.detect(&rgb)
    }
}

pub use baseline::HaarDetector;
pub use nms::nms;
