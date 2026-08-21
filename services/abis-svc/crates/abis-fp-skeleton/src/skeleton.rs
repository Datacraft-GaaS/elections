//! Zhang-Suen parallel thinning algorithm.
//!
//! Reference: T. Y. Zhang, C. Y. Suen, "A fast parallel algorithm for thinning digital patterns,"
//! Commun. ACM, vol. 28, no. 3, pp. 236-239, 1985.
//!
//! Iteratively removes contour pixels from the binary image in two sub-iterations
//! until no more pixels can be deleted.  The result is a 1-pixel-wide skeleton.

use crate::SkeletonError;

/// A binary (thresholded) image where 1 = foreground (ridge), 0 = background.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryImage {
    pub width: u32,
    pub height: u32,
    /// Row-major pixels: 1 = foreground, 0 = background.
    pub pixels: Vec<u8>,
}

impl BinaryImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, SkeletonError> {
        if pixels.len() != (width * height) as usize {
            return Err(SkeletonError::InvalidDimensions { width, height });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// Create from a grayscale image using a fixed threshold.
    pub fn from_gray(
        width: u32,
        height: u32,
        gray: &[u8],
        threshold: u8,
    ) -> Result<Self, SkeletonError> {
        if gray.len() != (width * height) as usize {
            return Err(SkeletonError::InvalidDimensions { width, height });
        }
        let pixels = gray
            .iter()
            .map(|&p| if p < threshold { 1u8 } else { 0u8 })
            .collect();
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return 0;
        }
        self.pixels[(y as u32 * self.width + x as u32) as usize]
    }

    #[inline]
    pub fn set(&mut self, x: u32, y: u32, v: u8) {
        self.pixels[(y * self.width + x) as usize] = v;
    }

    pub fn count_foreground(&self) -> usize {
        self.pixels.iter().filter(|&&v| v > 0).count()
    }
}

/// The thinned skeleton.  Pixel value 1 = skeleton pixel, 0 = background.
pub type RidgeSkeleton = BinaryImage;

/// Apply Zhang-Suen thinning to a binary image.
///
/// Returns a new BinaryImage with 1-pixel-wide ridge lines.
pub fn extract_skeleton(image: &BinaryImage) -> Result<RidgeSkeleton, SkeletonError> {
    if image.width < 8 || image.height < 8 {
        return Err(SkeletonError::ImageTooSmall);
    }
    let mut pixels = image.pixels.clone();
    let w = image.width as usize;
    let h = image.height as usize;

    loop {
        let mut changed = false;
        // Sub-iteration 1.
        // neighbours_8 handles out-of-bounds by returning 0, so start from 0
        // rather than 1 to allow border pixels to be thinning candidates.
        let mut to_delete: Vec<usize> = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                if pixels[idx] == 0 {
                    continue;
                }
                let p = neighbours_8(&pixels, w, x, y);
                if zhang_suen_condition_1(&p) && zhang_suen_condition_2(&p) && sub1_conditions(&p) {
                    to_delete.push(idx);
                }
            }
        }
        if !to_delete.is_empty() {
            changed = true;
            for idx in to_delete {
                pixels[idx] = 0;
            }
        }
        // Sub-iteration 2.
        let mut to_delete: Vec<usize> = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                if pixels[idx] == 0 {
                    continue;
                }
                let p = neighbours_8(&pixels, w, x, y);
                if zhang_suen_condition_1(&p) && zhang_suen_condition_2(&p) && sub2_conditions(&p) {
                    to_delete.push(idx);
                }
            }
        }
        if !to_delete.is_empty() {
            changed = true;
            for idx in to_delete {
                pixels[idx] = 0;
            }
        }
        if !changed {
            break;
        }
    }

    BinaryImage::new(image.width, image.height, pixels)
}

/// Return 8-neighbourhood in clockwise order: [P2..P9].
/// P2=N, P3=NE, P4=E, P5=SE, P6=S, P7=SW, P8=W, P9=NW.
fn neighbours_8(pixels: &[u8], w: usize, x: usize, y: usize) -> [u8; 8] {
    let g = |dx: i32, dy: i32| -> u8 {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= (pixels.len() / w) as i32 {
            0
        } else {
            pixels[ny as usize * w + nx as usize]
        }
    };
    [
        g(0, -1),  // P2
        g(1, -1),  // P3
        g(1, 0),   // P4
        g(1, 1),   // P5
        g(0, 1),   // P6
        g(-1, 1),  // P7
        g(-1, 0),  // P8
        g(-1, -1), // P9
    ]
}

/// Condition 1: 2 <= B(P1) <= 6 (number of ON neighbours).
fn zhang_suen_condition_1(p: &[u8; 8]) -> bool {
    let b: u8 = p.iter().sum();
    (2..=6).contains(&b)
}

/// Condition 2: A(P1) == 1 (exactly one 0->1 transition in the circular order).
fn zhang_suen_condition_2(p: &[u8; 8]) -> bool {
    let mut transitions = 0u8;
    for i in 0..8 {
        if p[i] == 0 && p[(i + 1) % 8] == 1 {
            transitions += 1;
        }
    }
    transitions == 1
}

/// Sub-iteration 1 conditions: P2*P4*P6 == 0 AND P4*P6*P8 == 0.
fn sub1_conditions(p: &[u8; 8]) -> bool {
    // p[0]=P2, p[2]=P4, p[4]=P6, p[6]=P8
    (p[0] * p[2] * p[4] == 0) && (p[2] * p[4] * p[6] == 0)
}

/// Sub-iteration 2 conditions: P2*P4*P8 == 0 AND P2*P6*P8 == 0.
fn sub2_conditions(p: &[u8; 8]) -> bool {
    (p[0] * p[2] * p[6] == 0) && (p[0] * p[4] * p[6] == 0)
}
