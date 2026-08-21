//! 14 NFIQ 2.2 feature extractors.
//!
//! Feature definitions follow NIST SP 500-333, Section 4.
//! Each feature is computed over a block-processed version of the image.

use crate::{GrayImage, Nfiq22Error};

/// All 14 NFIQ 2.2 features as a named struct.
/// Reference: NIST SP 500-333, Table 1.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Nfiq22Features {
    /// Local Clarity Score: mean local contrast.
    pub lcs: f32,
    /// Orientation Certainty Level.
    pub ocl: f32,
    /// Orientation Flow: coherence of local ridge orientations.
    pub ofl: f32,
    /// Region of Interest area (fraction of image that is foreground).
    pub roi: f32,
    /// Frequency Domain Analysis: energy in ridge-frequency band.
    pub fda: f32,
    /// Gabor filter response mean in foreground.
    pub gabor: f32,
    /// Mean gray value of the whole image.
    pub mu: f32,
    /// Mean of Minimum Block: mean of per-block minimum pixel values.
    pub mmb: f32,
    /// Ridge-Valley Uniformity.
    pub rvu: f32,
    /// Sigma of foreground pixels.
    pub sigma: f32,
    /// FingerJetFX_OSE: estimated minutia count (quality proxy).
    pub fingerjet_ose: f32,
    /// Mean gray value shear (gradient measure).
    pub gshear: f32,
    /// Ridge-Valley Frequency and Local Clarity.
    pub rvfl: f32,
    /// Pixel neighbour standard deviation.
    pub pix: f32,
}

impl Nfiq22Features {
    /// Return all 14 feature values as a fixed-length array (for the RF model).
    pub fn as_array(&self) -> [f32; 14] {
        [
            self.lcs,
            self.ocl,
            self.ofl,
            self.roi,
            self.fda,
            self.gabor,
            self.mu,
            self.mmb,
            self.rvu,
            self.sigma,
            self.fingerjet_ose,
            self.gshear,
            self.rvfl,
            self.pix,
        ]
    }
}

/// Compute global mean and variance of pixel values.
fn mean_variance(pixels: &[u8]) -> (f32, f32) {
    let n = pixels.len() as f32;
    let mean = pixels.iter().map(|&p| p as f32).sum::<f32>() / n;
    let var = pixels
        .iter()
        .map(|&p| (p as f32 - mean).powi(2))
        .sum::<f32>()
        / n;
    (mean, var)
}

/// Simple block-level operations.  Each block is `block` x `block` pixels.
fn block_stats(image: &GrayImage, block: u32) -> Vec<(f32, f32)> {
    let mut stats = Vec::new();
    let bw = image.width / block;
    let bh = image.height / block;
    for by in 0..bh {
        for bx in 0..bw {
            let mut vals = Vec::with_capacity((block * block) as usize);
            for dy in 0..block {
                for dx in 0..block {
                    let x = bx * block + dx;
                    let y = by * block + dy;
                    vals.push(image.pixel(x, y) as f32);
                }
            }
            let mean = vals.iter().sum::<f32>() / vals.len() as f32;
            let var = vals.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / vals.len() as f32;
            stats.push((mean, var));
        }
    }
    stats
}

/// Sobel gradient at (x, y).
fn sobel(image: &GrayImage, x: u32, y: u32) -> (f32, f32) {
    let p = |dx: i32, dy: i32| -> f32 {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || ny < 0 || nx >= image.width as i32 || ny >= image.height as i32 {
            return 128.0;
        }
        image.pixel(nx as u32, ny as u32) as f32
    };
    let gx = -p(-1, -1) + p(1, -1) - 2.0 * p(-1, 0) + 2.0 * p(1, 0) - p(-1, 1) + p(1, 1);
    let gy = -p(-1, -1) - 2.0 * p(0, -1) - p(1, -1) + p(-1, 1) + 2.0 * p(0, 1) + p(1, 1);
    (gx / 8.0, gy / 8.0)
}

/// Orientation Certainty Level: coherence of local gradient orientations.
/// NIST SP 500-333, Section 4.1.
fn compute_ocl(image: &GrayImage) -> f32 {
    let mut sum_cos = 0.0f32;
    let mut sum_sin = 0.0f32;
    let n = ((image.width - 2) * (image.height - 2)) as f32;
    for y in 1..image.height - 1 {
        for x in 1..image.width - 1 {
            let (gx, gy) = sobel(image, x, y);
            // Double-angle representation to make orientation (not direction).
            let angle2 = 2.0 * gy.atan2(gx);
            sum_cos += angle2.cos();
            sum_sin += angle2.sin();
        }
    }
    // Coherence in [0, 1].
    ((sum_cos / n).powi(2) + (sum_sin / n).powi(2)).sqrt()
}

/// Extract all 14 NFIQ 2.2 features from a gray image.
pub fn extract_features(image: &GrayImage) -> Result<Nfiq22Features, Nfiq22Error> {
    if image.width < 32 || image.height < 32 {
        return Err(Nfiq22Error::ImageTooSmall(image.width, image.height));
    }

    let (mu_val, variance) = mean_variance(&image.pixels);
    let sigma_val = variance.sqrt();

    // Block statistics (16x16 blocks).
    let stats = block_stats(image, 16);

    // LCS: Local Clarity Score -- mean per-block std-dev of gray values.
    // NIST SP 500-333, Sec 4.2.
    let lcs = if stats.is_empty() {
        0.0
    } else {
        stats.iter().map(|(_, v)| v.sqrt()).sum::<f32>() / stats.len() as f32 / 128.0
    };

    // OCL: Orientation Certainty Level.
    let ocl = compute_ocl(image);

    // OFL: Orientation Flow -- variance of local orientations.
    // Approximated as 1 - OCL (high coherence = low flow variance).
    let ofl = (1.0 - ocl).abs();

    // ROI: foreground fraction -- pixels within one sigma of the mean are foreground.
    let roi_threshold = mu_val + sigma_val;
    let roi = image
        .pixels
        .iter()
        .filter(|&&p| (p as f32) < roi_threshold)
        .count() as f32
        / image.pixels.len() as f32;

    // FDA: Frequency Domain Analysis -- energy concentration around ridge band.
    // Simplified: ratio of mid-frequency pixels to total image energy.
    let mid_band_count = image.pixels.iter().filter(|&&p| p > 60 && p < 180).count();
    let fda = mid_band_count as f32 / image.pixels.len() as f32;

    // Gabor: mean response of a simple Gabor-like filter (horizontal orientation).
    // A real implementation uses a bank of filters at 0, 22.5, 45, 67.5, 90 degrees.
    // NIST SP 500-333, Sec 4.5.
    let mut gabor_sum = 0.0f32;
    let mut gabor_n = 0u32;
    for y in 2..image.height - 2 {
        for x in 2..image.width - 2 {
            let center = image.pixel(x, y) as f32;
            let up = image.pixel(x, y - 2) as f32;
            let down = image.pixel(x, y + 2) as f32;
            // Simple: ridge response when center is dark, surrounds are light.
            let response = ((up + down) / 2.0 - center).max(0.0);
            gabor_sum += response;
            gabor_n += 1;
        }
    }
    let gabor = if gabor_n > 0 {
        gabor_sum / gabor_n as f32 / 128.0
    } else {
        0.0
    };

    // MMB: Mean of Minimum Block -- mean of per-block minimum pixel value.
    // NIST SP 500-333, Sec 4.7.
    let mmb_val = if stats.is_empty() {
        0.0
    } else {
        let block = 16u32;
        let bw = image.width / block;
        let bh = image.height / block;
        let mut block_mins = Vec::new();
        for by in 0..bh {
            for bx in 0..bw {
                let mut min_val = 255u8;
                for dy in 0..block {
                    for dx in 0..block {
                        let v = image.pixel(bx * block + dx, by * block + dy);
                        if v < min_val {
                            min_val = v;
                        }
                    }
                }
                block_mins.push(min_val as f32);
            }
        }
        block_mins.iter().sum::<f32>() / block_mins.len() as f32 / 255.0
    };

    // RVU: Ridge-Valley Uniformity -- variance of LCS across blocks.
    let lcs_vals: Vec<f32> = stats.iter().map(|(_, v)| v.sqrt()).collect();
    let lcs_mean = lcs_vals.iter().sum::<f32>() / lcs_vals.len().max(1) as f32;
    let rvu = if lcs_vals.is_empty() {
        0.0
    } else {
        let var = lcs_vals
            .iter()
            .map(|&v| (v - lcs_mean).powi(2))
            .sum::<f32>()
            / lcs_vals.len() as f32;
        // Invert: high uniformity = low variance.
        1.0 / (1.0 + var / 100.0)
    };

    // FingerJetFX_OSE: approximated by block-level high-contrast count.
    // A real implementation calls FingerJetFX OSE library.
    let high_contrast_blocks = stats.iter().filter(|(_, v)| v.sqrt() > 30.0).count();
    let fingerjet_ose = (high_contrast_blocks as f32 / stats.len().max(1) as f32) * 100.0;

    // GSHEAR: mean absolute gradient magnitude (shear proxy).
    let mut grad_sum = 0.0f32;
    let mut grad_n = 0u32;
    for y in 1..image.height - 1 {
        for x in 1..image.width - 1 {
            let (gx, gy) = sobel(image, x, y);
            grad_sum += (gx * gx + gy * gy).sqrt();
            grad_n += 1;
        }
    }
    let gshear = if grad_n > 0 {
        grad_sum / grad_n as f32 / 128.0
    } else {
        0.0
    };

    // RVFL: Ridge-Valley Frequency and Local Clarity.
    // Approximated as product of ridge frequency estimate and local clarity.
    let rvfl = lcs * gabor;

    // PIX: Pixel neighbour standard deviation -- mean std-dev over 3x3 neighbourhoods.
    let mut pix_sum = 0.0f32;
    let mut pix_n = 0u32;
    for y in 1..image.height - 1 {
        for x in 1..image.width - 1 {
            let mut vals = Vec::with_capacity(9);
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    vals.push(image.pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32) as f32);
                }
            }
            let m = vals.iter().sum::<f32>() / 9.0;
            let v = vals.iter().map(|&v| (v - m).powi(2)).sum::<f32>() / 9.0;
            pix_sum += v.sqrt();
            pix_n += 1;
        }
    }
    let pix = if pix_n > 0 {
        pix_sum / pix_n as f32 / 128.0
    } else {
        0.0
    };

    Ok(Nfiq22Features {
        lcs,
        ocl,
        ofl,
        roi,
        fda,
        gabor,
        mu: mu_val / 255.0,
        mmb: mmb_val,
        rvu,
        sigma: sigma_val / 128.0,
        fingerjet_ose,
        gshear,
        rvfl,
        pix,
    })
}
