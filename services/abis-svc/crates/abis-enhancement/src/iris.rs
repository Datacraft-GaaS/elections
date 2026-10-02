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

use crate::fingerprint::GrayscaleImage;
use crate::EnhancementError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrisCircle {
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusionMask {
    pub width: u32,
    pub height: u32,
    pub mask: Vec<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrisSegmentation {
    pub pupil: IrisCircle,
    pub limbus: IrisCircle,
    pub occlusion: OcclusionMask,
}

fn sample_circle_avg(img: &GrayscaleImage, cx: f32, cy: f32, radius: f32, n_samples: usize) -> f32 {
    let mut sum = 0.0f32;
    for k in 0..n_samples {
        let angle = k as f32 * 2.0 * std::f32::consts::PI / n_samples as f32;
        let x = (cx + radius * angle.cos()).clamp(0.0, (img.width - 1) as f32) as u32;
        let y = (cy + radius * angle.sin()).clamp(0.0, (img.height - 1) as f32) as u32;
        sum += img.pixel(x, y) as f32;
    }
    if n_samples == 0 {
        128.0
    } else {
        sum / n_samples as f32
    }
}

pub fn integro_differential_operator(
    img: &GrayscaleImage,
    r_min: u32,
    r_max: u32,
) -> Result<IrisCircle, EnhancementError> {
    let w = img.width;
    let h = img.height;
    if w < 20 || h < 20 {
        return Err(EnhancementError::InvalidDimensions {
            width: w,
            height: h,
        });
    }
    let mut best_contrast = f32::NEG_INFINITY;
    let mut best = IrisCircle {
        center_x: w as f32 / 2.0,
        center_y: h as f32 / 2.0,
        radius: r_min as f32,
    };
    let step_xy = 5u32;
    let step_r = 2u32;
    let mut cy = 0u32;
    while cy < h {
        let mut cx = 0u32;
        while cx < w {
            let mut r = r_min;
            while r < r_max {
                let inner_r = r.saturating_sub(2);
                let outer_r = r + 2;
                if outer_r as f32 > (w.min(h) as f32 / 2.0) {
                    r += step_r;
                    continue;
                }
                let inner_avg = sample_circle_avg(img, cx as f32, cy as f32, inner_r as f32, 36);
                let outer_avg = sample_circle_avg(img, cx as f32, cy as f32, outer_r as f32, 36);
                let contrast = (outer_avg - inner_avg).abs();
                if contrast > best_contrast {
                    best_contrast = contrast;
                    best = IrisCircle {
                        center_x: cx as f32,
                        center_y: cy as f32,
                        radius: r as f32,
                    };
                }
                r += step_r;
            }
            cx += step_xy;
        }
        cy += step_xy;
    }
    Ok(best)
}

pub fn detect_eyelid_occlusion(
    img: &GrayscaleImage,
    limbus: &IrisCircle,
) -> Result<OcclusionMask, EnhancementError> {
    let lx = (limbus.center_x - limbus.radius).max(0.0) as u32;
    let ly = (limbus.center_y - limbus.radius).max(0.0) as u32;
    let rx = (limbus.center_x + limbus.radius).min((img.width - 1) as f32) as u32;
    let ry = (limbus.center_y + limbus.radius).min((img.height - 1) as f32) as u32;
    let bw = rx.saturating_sub(lx);
    let bh = ry.saturating_sub(ly);
    let band_h = (bh as f32 * 0.15).round() as u32;
    let mut mask = vec![false; (bw * bh) as usize];
    for row in 0..bh {
        for col in 0..bw {
            let in_top = row < band_h;
            let in_bottom = row >= bh.saturating_sub(band_h);
            mask[(row * bw + col) as usize] = in_top || in_bottom;
        }
    }
    Ok(OcclusionMask {
        width: bw,
        height: bh,
        mask,
    })
}

pub fn segment_iris(img: &GrayscaleImage) -> Result<IrisSegmentation, EnhancementError> {
    if img.width < 20 || img.height < 20 {
        return Err(EnhancementError::InvalidDimensions {
            width: img.width,
            height: img.height,
        });
    }
    let pupil = integro_differential_operator(img, 10, 80)?;
    let r_max = img.width.min(img.height) / 2;
    let limbus = integro_differential_operator(img, 80, r_max)?;
    let occlusion = detect_eyelid_occlusion(img, &limbus)?;
    Ok(IrisSegmentation {
        pupil,
        limbus,
        occlusion,
    })
}
