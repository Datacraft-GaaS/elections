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

//! Individual ISO/IEC 29794-5 quality metrics.
//!
//! All metrics return f32 in [0.0, 1.0]. Higher is better.

use crate::{QualityError, QualityLandmarks};

// ---------------------------------------------------------------------------
// Sharpness (Laplacian variance)
// Reference: Pertuz et al. (2013). "Analysis of focus measure operators for
//   shape-from-focus." Pattern Recognition 46(5):1415-1432. Table 1, LAPV.
// ---------------------------------------------------------------------------

/// Compute sharpness from Laplacian variance on a grayscale image.
///
/// Returns a normalised score in [0.0, 1.0]. The raw Laplacian variance is
/// clamped to `max_var` (default 500) before normalisation.
pub fn sharpness(gray: &[u8], width: u32, height: u32) -> Result<f32, QualityError> {
    if width < 32 || height < 32 {
        return Err(QualityError::ImageTooSmall);
    }
    let w = width as usize;
    let h = height as usize;

    // Discrete 3x3 Laplacian kernel: centre = -4, 4 neighbours = +1.
    let mut sum = 0f64;
    let mut sum_sq = 0f64;
    let mut count = 0u64;

    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            let c = gray[y * w + x] as f64;
            let t = gray[(y - 1) * w + x] as f64;
            let b = gray[(y + 1) * w + x] as f64;
            let l = gray[y * w + (x - 1)] as f64;
            let r = gray[y * w + (x + 1)] as f64;
            let lap = t + b + l + r - 4.0 * c;
            sum += lap;
            sum_sq += lap * lap;
            count += 1;
        }
    }

    if count == 0 {
        return Ok(0.0);
    }
    let mean = sum / count as f64;
    let variance = (sum_sq / count as f64 - mean * mean).max(0.0);
    // Normalise: 500 is empirically the 99th-percentile for sharp passport photos.
    let max_var = 500.0f64;
    Ok((variance / max_var).min(1.0) as f32)
}

// ---------------------------------------------------------------------------
// Resolution: interocular distance in pixels
// Reference: ISO/IEC 19794-5:2011 clause 7.2.1 requires interocular distance
//   >= 90 pixels for standard enrolment images.
// ---------------------------------------------------------------------------

/// Interocular distance normalised by the ISO/IEC 19794-5 minimum of 90px.
/// Score = clamp(iod / 90, 0, 1). Images with iod >= 90px score 1.0.
pub fn resolution_interocular(lm: &QualityLandmarks) -> Result<f32, QualityError> {
    let iod = lm.interocular_distance();
    if iod < 1e-3 {
        return Err(QualityError::ZeroInterocularDistance);
    }
    Ok((iod / 90.0).clamp(0.0, 1.0))
}

// ---------------------------------------------------------------------------
// Pose: yaw + pitch + roll from landmark geometry
// Reference: Murphy-Chutorian & Trivedi (2009). "Head Pose Estimation in
//   Computer Vision: A Survey." IEEE TPAMI 31(4):607-626.
// ---------------------------------------------------------------------------

/// Estimated head pose angles (degrees) from 5-point landmarks.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PoseAngles {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

/// Estimate head pose from 5-point landmarks using geometric heuristics.
///
/// Yaw: left-right asymmetry of eye-to-nose horizontal offsets.
/// Pitch: ratio of eye-midpoint to nose vertical distance vs. face height.
/// Roll: arctangent of inter-eye vertical offset vs. horizontal distance.
pub fn pose_angles(lm: &QualityLandmarks) -> PoseAngles {
    let iod = lm.interocular_distance().max(1.0);

    // Roll: angle of the inter-eye line from horizontal.
    let dx = lm.right_eye.0 - lm.left_eye.0;
    let dy = lm.right_eye.1 - lm.left_eye.1;
    let roll = dy.atan2(dx).to_degrees();

    // Yaw: left vs. right eye to nose horizontal distance ratio.
    let d_left = (lm.nose.0 - lm.left_eye.0).abs();
    let d_right = (lm.right_eye.0 - lm.nose.0).abs();
    let asym = (d_right - d_left) / iod;
    // Empirical mapping: 0.3 asymmetry ~ 30 degrees yaw.
    let yaw = (asym * 100.0).clamp(-90.0, 90.0);

    // Pitch: nose y relative to eye midpoint and mouth midpoint.
    let eye_mid_y = (lm.left_eye.1 + lm.right_eye.1) / 2.0;
    let mouth_mid_y = (lm.left_mouth.1 + lm.right_mouth.1) / 2.0;
    let face_h = (mouth_mid_y - eye_mid_y).max(1.0);
    let nose_ratio = (lm.nose.1 - eye_mid_y) / face_h;
    // nose_ratio ~ 0.5 for frontal; deviation maps to pitch.
    let pitch = ((nose_ratio - 0.5) * 120.0).clamp(-60.0, 60.0);

    PoseAngles { yaw, pitch, roll }
}

/// Pose quality score: 1.0 when all angles near zero; degrades linearly.
///
/// Policy limits (ISO 29794-5): yaw < 10 deg, pitch < 15 deg, roll < 8 deg.
pub fn pose_quality(angles: &PoseAngles) -> f32 {
    let yaw_score = (1.0 - angles.yaw.abs() / 45.0).clamp(0.0, 1.0);
    let pitch_score = (1.0 - angles.pitch.abs() / 45.0).clamp(0.0, 1.0);
    let roll_score = (1.0 - angles.roll.abs() / 45.0).clamp(0.0, 1.0);
    (yaw_score * pitch_score * roll_score).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Illumination: mean intensity + histogram spread
// Reference: ISO/IEC 29794-5 clause 6.4 (illumination).
// ---------------------------------------------------------------------------

/// Illumination score from mean intensity and histogram range.
///
/// Ideal: mean in [100, 180], spread (max - min) > 50.
pub fn illumination(gray: &[u8]) -> f32 {
    if gray.is_empty() {
        return 0.0;
    }
    let mean: f32 = gray.iter().map(|&v| v as f32).sum::<f32>() / gray.len() as f32;
    let max_v = gray.iter().copied().max().unwrap_or(0) as f32;
    let min_v = gray.iter().copied().min().unwrap_or(0) as f32;
    let spread = (max_v - min_v) / 255.0;

    // Mean score: penalise very dark or very bright images.
    let mean_score = if mean < 50.0 {
        mean / 50.0
    } else if mean > 200.0 {
        (255.0 - mean) / 55.0
    } else {
        1.0
    };
    ((mean_score + spread) / 2.0).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Uniformity: background standard deviation
// Reference: ISO/IEC 29794-5 clause 6.3 (background).
// ---------------------------------------------------------------------------

/// Uniformity score based on std-dev of the border-ring pixels (background
/// proxy). Lower std-dev => more uniform background => higher score.
///
/// Score = 1 - clamp(std_dev / 60, 0, 1).
pub fn uniformity(gray: &[u8], width: u32, height: u32) -> f32 {
    if gray.is_empty() || width < 4 || height < 4 {
        return 0.0;
    }
    let w = width as usize;
    let h = height as usize;
    let border = 8.min(w / 4).min(h / 4).max(1);

    // Sample top + bottom rows and left + right columns.
    let mut samples: Vec<f32> = Vec::new();
    for x in 0..w {
        for b in 0..border {
            samples.push(gray[b * w + x] as f32);
            samples.push(gray[(h - 1 - b) * w + x] as f32);
        }
    }
    for y in border..(h - border) {
        for b in 0..border {
            samples.push(gray[y * w + b] as f32);
            samples.push(gray[y * w + (w - 1 - b)] as f32);
        }
    }
    if samples.is_empty() {
        return 1.0;
    }
    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    let var: f32 = samples
        .iter()
        .map(|&v| (v - mean) * (v - mean))
        .sum::<f32>()
        / samples.len() as f32;
    let std_dev = var.sqrt();
    (1.0 - (std_dev / 60.0).clamp(0.0, 1.0)).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Occlusion: skin-tone coverage percentage on the aligned template
// Reference: Klum et al. (2014). "Soft biometric attributes from facial
//   shape." IEEE BTAS 2014. (skin-tone segmentation approach).
// ---------------------------------------------------------------------------

/// Occlusion score: fraction of the face region covered by skin-tone pixels.
///
/// Skin-tone heuristic on RGB: R > G > B, R in [80, 240], R-B > 15.
/// Score = fraction of skin-tone pixels (higher = less occluded = better).
pub fn occlusion_coverage(rgb: &[u8], width: u32, height: u32) -> f32 {
    if rgb.is_empty() || width == 0 || height == 0 {
        return 0.0;
    }
    let total = (width * height) as usize;
    let mut skin = 0usize;
    for chunk in rgb.chunks(3) {
        if chunk.len() < 3 {
            break;
        }
        let r = chunk[0] as i32;
        let g = chunk[1] as i32;
        let b = chunk[2] as i32;
        if r > 80 && r < 240 && r > g && g > b && (r - b) > 15 {
            skin += 1;
        }
    }
    (skin as f32 / total as f32).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Expression neutrality: mouth-openness + smile strength from landmarks
// Reference: Shan, C. et al. (2009). "Facial expression recognition based on
//   local binary patterns." Pattern Recognition 42(7):1071-1079.
// ---------------------------------------------------------------------------

/// Expression neutrality score from mouth-landmark geometry.
///
/// Mouth openness = (left_mouth_y - right_mouth_y).abs() / iod.
/// Smile = horizontal mouth width relative to iod.
/// Score 1.0 = closed, neutral mouth; 0.0 = open or wide smile.
pub fn expression_neutrality(lm: &QualityLandmarks) -> f32 {
    let iod = lm.interocular_distance().max(1.0);

    // Mouth openness from vertical asymmetry of mouth corners.
    let mouth_v_diff = (lm.left_mouth.1 - lm.right_mouth.1).abs();
    let openness = (mouth_v_diff / iod).clamp(0.0, 1.0);

    // Mouth width relative to iod (wider = bigger smile).
    let mouth_w = (lm.right_mouth.0 - lm.left_mouth.0).abs();
    let width_ratio = (mouth_w / iod).clamp(0.0, 2.0) / 2.0;

    let smile_score = 1.0 - width_ratio.clamp(0.0, 1.0);
    ((1.0 - openness) * 0.5 + smile_score * 0.5).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Gaze: eye-corner-to-pupil offset
// Reference: Daugman, J. (2004). "How Iris Recognition Works." IEEE TCSVT.
//   (gaze normalisation principle)
// ---------------------------------------------------------------------------

/// Gaze quality score from pupil offsets relative to eye corners.
///
/// Requires pupil centre positions. If absent, returns 1.0 (no penalty).
/// Score decreases linearly as the pupil moves away from the eye-corner
/// midpoint (looking away from camera).
pub fn gaze_offset(lm: &QualityLandmarks) -> f32 {
    let (lp, rp) = match (lm.left_pupil, lm.right_pupil) {
        (Some(l), Some(r)) => (l, r),
        _ => return 1.0,
    };
    let iod = lm.interocular_distance().max(1.0);

    // Left eye: offset from eye corner to pupil, normalised by iod / 2.
    let le_mid = (lm.left_eye.0, lm.left_eye.1);
    let l_off = (((lp.0 - le_mid.0).powi(2) + (lp.1 - le_mid.1).powi(2)).sqrt()) / (iod / 2.0);

    let re_mid = (lm.right_eye.0, lm.right_eye.1);
    let r_off = (((rp.0 - re_mid.0).powi(2) + (rp.1 - re_mid.1).powi(2)).sqrt()) / (iod / 2.0);

    let avg_offset = (l_off + r_off) / 2.0;
    (1.0 - avg_offset.clamp(0.0, 1.0)).clamp(0.0, 1.0)
}
