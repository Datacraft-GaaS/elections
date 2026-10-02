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

//! Bicubic + unsharp mask stub. Production wires a super-resolution neural model (e.g. ESRGAN) via ONNX Runtime.

use crate::fingerprint::GrayscaleImage;
use crate::EnhancementError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperResConfig {
    pub scale_factor: u32,
    pub unsharp_radius: u32,
    pub unsharp_amount: f32,
}

impl Default for SuperResConfig {
    fn default() -> Self {
        Self {
            scale_factor: 2,
            unsharp_radius: 1,
            unsharp_amount: 0.5,
        }
    }
}

pub fn bicubic_upsample(
    img: &GrayscaleImage,
    scale: u32,
) -> Result<GrayscaleImage, EnhancementError> {
    if scale == 0 {
        return Err(EnhancementError::InvalidParameter(
            "scale must be > 0".into(),
        ));
    }
    let new_w = img.width * scale;
    let new_h = img.height * scale;
    let mut out = vec![0u8; (new_w * new_h) as usize];
    for oy in 0..new_h {
        for ox in 0..new_w {
            let sx = (ox / scale).min(img.width - 1);
            let sy = (oy / scale).min(img.height - 1);
            let sx1 = (sx + 1).min(img.width - 1);
            let sy1 = (sy + 1).min(img.height - 1);
            let fx = (ox % scale) as f32 / scale as f32;
            let fy = (oy % scale) as f32 / scale as f32;
            let p00 = img.pixel(sx, sy) as f32;
            let p10 = img.pixel(sx1, sy) as f32;
            let p01 = img.pixel(sx, sy1) as f32;
            let p11 = img.pixel(sx1, sy1) as f32;
            let val = p00 * (1.0 - fx) * (1.0 - fy)
                + p10 * fx * (1.0 - fy)
                + p01 * (1.0 - fx) * fy
                + p11 * fx * fy;
            out[(oy * new_w + ox) as usize] = val.clamp(0.0, 255.0).round() as u8;
        }
    }
    GrayscaleImage::new(new_w, new_h, out)
}

pub fn unsharp_mask(
    img: &GrayscaleImage,
    radius: u32,
    amount: f32,
) -> Result<GrayscaleImage, EnhancementError> {
    let w = img.width as usize;
    let h = img.height as usize;
    let kernel_size = (2 * radius + 1) as i32;
    let mut blurred = vec![0.0f32; w * h];
    let count = (kernel_size * kernel_size) as f32;
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f32;
            for dy in -(radius as i32)..=(radius as i32) {
                for dx in -(radius as i32)..=(radius as i32) {
                    let ny = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                    let nx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                    sum += img.pixels[ny * w + nx] as f32;
                }
            }
            blurred[y * w + x] = sum / count;
        }
    }
    let output: Vec<u8> = img
        .pixels
        .iter()
        .enumerate()
        .map(|(i, &orig)| {
            let o = orig as f32;
            let b = blurred[i];
            (o + amount * (o - b)).clamp(0.0, 255.0).round() as u8
        })
        .collect();
    GrayscaleImage::new(img.width, img.height, output)
}

pub fn super_resolve(
    img: &GrayscaleImage,
    config: &SuperResConfig,
) -> Result<GrayscaleImage, EnhancementError> {
    let upsampled = bicubic_upsample(img, config.scale_factor)?;
    unsharp_mask(&upsampled, config.unsharp_radius, config.unsharp_amount)
}
