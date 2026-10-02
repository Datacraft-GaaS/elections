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

//! Production emits per-pixel attention weights from the face model's self-attention heads.

use crate::ExplainError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionMap {
    pub width: u32,
    pub height: u32,
    pub values: Vec<f32>,
}

pub trait AttentionEmitter {
    fn emit_attention(
        &self,
        image_width: u32,
        image_height: u32,
    ) -> Result<AttentionMap, ExplainError>;
}

pub struct UniformAttentionEmitter;

impl AttentionEmitter for UniformAttentionEmitter {
    fn emit_attention(
        &self,
        image_width: u32,
        image_height: u32,
    ) -> Result<AttentionMap, ExplainError> {
        let total = image_width * image_height;
        if total == 0 {
            return Err(ExplainError::InvalidInput(
                "image dimensions must be nonzero".into(),
            ));
        }
        let val = 1.0 / total as f32;
        Ok(AttentionMap {
            width: image_width,
            height: image_height,
            values: vec![val; total as usize],
        })
    }
}
