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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FingerPosition {
    RightIndex = 2,
    RightMiddle = 3,
    RightRing = 4,
    RightLittle = 5,
    LeftIndex = 7,
    LeftMiddle = 8,
    LeftRing = 9,
    LeftLittle = 10,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Hand {
    Right,
    Left,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedFinger {
    pub position: FingerPosition,
    pub bbox: BoundingBox,
    pub image: GrayscaleImage,
}

pub fn segment_slap(
    slap: &GrayscaleImage,
    hand: Hand,
) -> Result<[SegmentedFinger; 4], EnhancementError> {
    if slap.width < 4 {
        return Err(EnhancementError::SegmentationFailed(format!(
            "slap image too narrow: width {} < 4",
            slap.width
        )));
    }
    let strip_w = slap.width / 4;
    let h = slap.height;
    let positions: [FingerPosition; 4] = match hand {
        Hand::Right => [
            FingerPosition::RightIndex,
            FingerPosition::RightMiddle,
            FingerPosition::RightRing,
            FingerPosition::RightLittle,
        ],
        Hand::Left => [
            FingerPosition::LeftLittle,
            FingerPosition::LeftRing,
            FingerPosition::LeftMiddle,
            FingerPosition::LeftIndex,
        ],
    };
    let mut fingers: Vec<SegmentedFinger> = Vec::with_capacity(4);
    for i in 0..4usize {
        let x_start = i as u32 * strip_w;
        let mut strip_pixels = vec![0u8; (strip_w * h) as usize];
        for y in 0..h {
            for x in 0..strip_w {
                let src_x = x_start + x;
                strip_pixels[(y * strip_w + x) as usize] = slap.pixel(src_x, y);
            }
        }
        let image = GrayscaleImage::new(strip_w, h, strip_pixels)
            .map_err(|e| EnhancementError::SegmentationFailed(format!("{e}")))?;
        fingers.push(SegmentedFinger {
            position: positions[i],
            bbox: BoundingBox {
                x: x_start,
                y: 0,
                w: strip_w,
                h,
            },
            image,
        });
    }
    let arr = [
        fingers.remove(0),
        fingers.remove(0),
        fingers.remove(0),
        fingers.remove(0),
    ];
    Ok(arr)
}
