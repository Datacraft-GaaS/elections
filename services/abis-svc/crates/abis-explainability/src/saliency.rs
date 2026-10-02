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

//! Vanilla Grad approximation: in production, wire automatic differentiation
//! from the face embedding model to get true gradient norms.

use crate::ExplainError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingGradient {
    pub region_x: u32,
    pub region_y: u32,
    pub region_w: u32,
    pub region_h: u32,
    pub gradient_magnitude: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaliencyMap {
    pub width: u32,
    pub height: u32,
    pub regions: Vec<EmbeddingGradient>,
}

pub fn compute_saliency(
    embedding_a: &[f32],
    embedding_b: &[f32],
    image_width: u32,
    image_height: u32,
    grid_size: u32,
) -> Result<SaliencyMap, ExplainError> {
    if embedding_a.is_empty() || embedding_b.is_empty() {
        return Err(ExplainError::InvalidInput(
            "embeddings must not be empty".into(),
        ));
    }

    let cell_w = image_width / grid_size;
    let cell_h = image_height / grid_size;
    let mut regions = Vec::with_capacity((grid_size * grid_size) as usize);

    for row in 0..grid_size {
        for col in 0..grid_size {
            let r = (row * grid_size + col) as usize;
            let mag =
                (embedding_a[r % embedding_a.len()] - embedding_b[r % embedding_b.len()]).abs();
            regions.push(EmbeddingGradient {
                region_x: col * cell_w,
                region_y: row * cell_h,
                region_w: cell_w,
                region_h: cell_h,
                gradient_magnitude: mag,
            });
        }
    }

    Ok(SaliencyMap {
        width: image_width,
        height: image_height,
        regions,
    })
}
