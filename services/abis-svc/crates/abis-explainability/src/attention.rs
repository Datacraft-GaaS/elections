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
