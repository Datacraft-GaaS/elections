use crate::EnhancementError;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrayscaleImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl GrayscaleImage {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Result<Self, EnhancementError> {
        let expected = (width * height) as usize;
        if data.len() != expected {
            return Err(EnhancementError::InvalidDimensions { width, height });
        }
        Ok(Self {
            width,
            height,
            pixels: data,
        })
    }

    pub fn pixel(&self, x: u32, y: u32) -> u8 {
        self.pixels[(y * self.width + x) as usize]
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, v: u8) {
        self.pixels[(y * self.width + x) as usize] = v;
    }

    pub fn clone_data(&self) -> Vec<u8> {
        self.pixels.clone()
    }
}

pub fn histogram_equalize(img: &GrayscaleImage) -> Result<GrayscaleImage, EnhancementError> {
    let total = img.pixels.len();
    if total == 0 {
        return Err(EnhancementError::InvalidDimensions {
            width: img.width,
            height: img.height,
        });
    }
    let mut hist = [0u64; 256];
    for &p in &img.pixels {
        hist[p as usize] += 1;
    }
    let mut cdf = [0u64; 256];
    cdf[0] = hist[0];
    for i in 1..256 {
        cdf[i] = cdf[i - 1] + hist[i];
    }
    let cdf_min = cdf.iter().copied().find(|&v| v > 0).unwrap_or(0);
    let denom = (total as u64).saturating_sub(cdf_min);
    let new_pixels: Vec<u8> = img
        .pixels
        .iter()
        .map(|&p| {
            if denom == 0 {
                0
            } else {
                let numerator = cdf[p as usize].saturating_sub(cdf_min);
                ((255.0 * numerator as f64 / denom as f64).round() as u64).min(255) as u8
            }
        })
        .collect();
    GrayscaleImage::new(img.width, img.height, new_pixels)
}

pub fn gabor_filter(
    img: &GrayscaleImage,
    frequency: f64,
    orientation: f64,
    sigma_x: f64,
    sigma_y: f64,
) -> Result<GrayscaleImage, EnhancementError> {
    const KERNEL_SIZE: usize = 15;
    const HALF: i32 = 7;
    let mut kernel = [[0.0f64; KERNEL_SIZE]; KERNEL_SIZE];
    for i in 0..KERNEL_SIZE {
        for j in 0..KERNEL_SIZE {
            let di = i as i32 - HALF;
            let dj = j as i32 - HALF;
            let df = di as f64;
            let djf = dj as f64;
            let x_prime = df * orientation.cos() + djf * orientation.sin();
            let y_prime = -df * orientation.sin() + djf * orientation.cos();
            let gauss = (-(x_prime.powi(2) / (2.0 * sigma_x.powi(2))
                + y_prime.powi(2) / (2.0 * sigma_y.powi(2))))
            .exp();
            let wave = (2.0
                * PI
                * frequency
                * (x_prime * orientation.cos() + y_prime * orientation.sin()))
            .cos();
            kernel[i][j] = gauss * wave;
        }
    }
    // Normalize by kernel sum so a constant image maps to itself (DC gain = 1).
    let k_sum: f64 = kernel.iter().flat_map(|row| row.iter()).copied().sum();
    if k_sum.abs() > 1e-12 {
        for row in kernel.iter_mut() {
            for v in row.iter_mut() {
                *v /= k_sum;
            }
        }
    }
    let w = img.width as usize;
    let h = img.height as usize;
    let mut output = vec![128u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f64;
            let mut in_border = false;
            'outer: for ki in 0..KERNEL_SIZE {
                for kj in 0..KERNEL_SIZE {
                    let sy = y as i32 + ki as i32 - HALF;
                    let sx = x as i32 + kj as i32 - HALF;
                    if sy < 0 || sy >= h as i32 || sx < 0 || sx >= w as i32 {
                        in_border = true;
                        break 'outer;
                    }
                    sum += kernel[ki][kj] * img.pixel(sx as u32, sy as u32) as f64;
                }
            }
            if !in_border {
                output[y * w + x] = sum.clamp(0.0, 255.0).round() as u8;
            }
        }
    }
    GrayscaleImage::new(img.width, img.height, output)
}

pub fn apply_gabor_bank(img: &GrayscaleImage) -> Result<GrayscaleImage, EnhancementError> {
    let n_orientations = 8usize;
    let w = img.width as usize;
    let h = img.height as usize;
    let mut sum_pixels = vec![0.0f64; w * h];
    for k in 0..n_orientations {
        let orientation = k as f64 * PI / n_orientations as f64;
        let filtered = gabor_filter(img, 0.1, orientation, 3.0, 5.0)?;
        for (i, &p) in filtered.pixels.iter().enumerate() {
            sum_pixels[i] += p as f64;
        }
    }
    let averaged: Vec<u8> = sum_pixels
        .iter()
        .map(|&s| (s / n_orientations as f64).clamp(0.0, 255.0).round() as u8)
        .collect();
    GrayscaleImage::new(img.width, img.height, averaged)
}

/// Simplified morphological thinning via erosion; Hilditch or Zhang-Suen thinning should replace this for production.
pub fn morphological_thin(
    img: &GrayscaleImage,
    threshold: u8,
) -> Result<GrayscaleImage, EnhancementError> {
    let w = img.width as usize;
    let h = img.height as usize;
    let binary: Vec<u8> = img
        .pixels
        .iter()
        .map(|&p| if p >= threshold { 255 } else { 0 })
        .collect();
    let erode = |input: &[u8]| -> Vec<u8> {
        let mut out = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                let mut min_val = 255u8;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let ny = y as i32 + dy;
                        let nx = x as i32 + dx;
                        if ny >= 0 && ny < h as i32 && nx >= 0 && nx < w as i32 {
                            let v = input[(ny as usize) * w + nx as usize];
                            if v < min_val {
                                min_val = v;
                            }
                        }
                    }
                }
                out[y * w + x] = min_val;
            }
        }
        out
    };
    let pass1 = erode(&binary);
    let pass2 = erode(&pass1);
    GrayscaleImage::new(img.width, img.height, pass2)
}

pub fn restore(img: &GrayscaleImage) -> Result<GrayscaleImage, EnhancementError> {
    let equalized = histogram_equalize(img)?;
    let filtered = apply_gabor_bank(&equalized)?;
    morphological_thin(&filtered, 128)
}
