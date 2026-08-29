//! Iris segmentation: Daugman integro-differential operator + OSIRIS boundary refinement.
//!
//! References:
//!   Daugman, J. (1993). High confidence visual recognition of persons by a test of statistical independence.
//!   IEEE Transactions on Pattern Analysis and Machine Intelligence, 15(11), 1148-1161.
//!
//!   Othman, N., Dorizzi, B., & Garcia-Salicetti, S. (2016). OSIRIS: An open source iris recognition software.
//!   Pattern Recognition Letters, 82, 124-131.
#![warn(missing_docs)]

pub mod daugman;
pub mod mask;
pub mod osiris;
pub mod segment;

pub use segment::{IrisSegmentation, SegmentIris, SegmentError};

/// A minimal grayscale image (row-major, 8-bit per pixel).
#[derive(Debug, Clone)]
pub struct GrayImage {
    pub width: u32,
    pub height: u32,
    data: Vec<u8>,
}

impl GrayImage {
    /// Construct from raw pixel bytes (row-major, 8 bpp).
    ///
    /// # Errors
    /// Returns `None` when `data.len() != width * height`.
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Option<Self> {
        if data.len() as u64 != width as u64 * height as u64 {
            return None;
        }
        Some(Self { width, height, data })
    }

    /// Construct from raw bytes, panicking on size mismatch (test use).
    pub fn from_raw(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self::new(width, height, data).expect("data length must equal width * height")
    }

    /// Pixel value at (x, y), or 0 if out of bounds.
    #[inline]
    pub fn get(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        self.data[(y * self.width + x) as usize]
    }

    /// Pixel value at (x, y) as f32.
    #[inline]
    pub fn get_f32(&self, x: u32, y: u32) -> f32 {
        self.get(x, y) as f32
    }

    /// Raw pixel slice.
    pub fn pixels(&self) -> &[u8] {
        &self.data
    }

    /// Pixel count.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// A circle defined by centre and radius.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Circle {
    pub cx: f32,
    pub cy: f32,
    pub radius: f32,
}

impl Circle {
    pub fn new(cx: f32, cy: f32, radius: f32) -> Self {
        Self { cx, cy, radius }
    }

    /// True when (x, y) is inside the circle.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        let dx = x - self.cx;
        let dy = y - self.cy;
        dx * dx + dy * dy <= self.radius * self.radius
    }
}

/// Binary occlusion mask (1 = iris pixel usable, 0 = occluded/specular).
///
/// Stored as a flat Vec<bool>, row-major, same dimensions as the source image.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IrisMask {
    pub width: u32,
    pub height: u32,
    bits: Vec<bool>,
}

impl IrisMask {
    pub fn all_valid(width: u32, height: u32) -> Self {
        Self { width, height, bits: vec![true; (width * height) as usize] }
    }

    pub fn zeroed(width: u32, height: u32) -> Self {
        Self { width, height, bits: vec![false; (width * height) as usize] }
    }

    pub fn set(&mut self, x: u32, y: u32, val: bool) {
        if x < self.width && y < self.height {
            self.bits[(y * self.width + x) as usize] = val;
        }
    }

    pub fn get(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.bits[(y * self.width + x) as usize]
    }

    /// Fraction of pixels marked valid.
    pub fn valid_fraction(&self) -> f32 {
        if self.bits.is_empty() {
            return 0.0;
        }
        self.bits.iter().filter(|&&b| b).count() as f32 / self.bits.len() as f32
    }

    /// Fraction of pixels marked occluded.
    pub fn occlusion_fraction(&self) -> f32 {
        1.0 - self.valid_fraction()
    }

    /// Merge another mask: result is valid only where both are valid.
    pub fn and(&self, other: &Self) -> Self {
        let bits = self.bits.iter().zip(other.bits.iter()).map(|(&a, &b)| a && b).collect();
        Self { width: self.width, height: self.height, bits }
    }
}
