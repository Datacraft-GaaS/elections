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

//! Face embedding: ArcFace/AdaFace/CosFace interface.
//!
//! References:
//! - Deng, J. et al. (2019). "ArcFace: Additive Angular Margin Loss for Deep
//!   Face Recognition." CVPR 2019. (cosine margin loss)
//! - Kim, M. et al. (2022). "AdaFace: Quality Adaptive Margin for Face
//!   Recognition." CVPR 2022. (adaptive margin based on image quality)
//! - Wang, H. et al. (2018). "CosFace: Large Margin Cosine Loss for Deep Face
//!   Recognition." CVPR 2018. (cosine softmax margin)
#![warn(missing_docs)]

pub mod hash_embedder;
pub mod model;

pub use model::{EmbeddingModel, LossFamily};
pub use hash_embedder::HashEmbedder;

use thiserror::Error;

/// Embedding dimension (512 for ArcFace/AdaFace/CosFace standard models).
pub const EMBED_DIM: usize = 512;

/// Embedding error.
#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("input image is empty or has wrong dimensions")]
    InvalidInput,
    #[error("model inference failed: {0}")]
    InferenceFailed(String),
}

/// Face embedder trait: maps an aligned face image to an L2-normalised vector.
///
/// The aligned image is expected to be 112x112 RGB (3 bytes per pixel,
/// row-major), matching the output of `eel-abis-face-align`.
pub trait FaceEmbedder {
    /// Embed an aligned 112x112 RGB image.
    fn embed(&self, aligned_rgb: &[u8]) -> Result<[f32; EMBED_DIM], EmbedError>;

    /// Convenience: embed and return as Vec<f32>.
    fn embed_vec(&self, aligned_rgb: &[u8]) -> Result<Vec<f32>, EmbedError> {
        Ok(self.embed(aligned_rgb)?.to_vec())
    }
}

/// L2-normalise a mutable 512-d embedding in-place.
///
/// Leaves the vector unchanged if the norm is below the epsilon threshold
/// (avoids dividing by zero for the degenerate all-zero case).
pub fn l2_normalise(v: &mut [f32; EMBED_DIM]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm < 1e-9 {
        return;
    }
    for x in v.iter_mut() {
        *x /= norm;
    }
}

/// L2-normalise a slice, returning a new array.
pub fn l2_normalise_slice(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm < 1e-9 {
        return v.to_vec();
    }
    v.iter().map(|x| x / norm).collect()
}

/// Cosine similarity between two L2-normalised embeddings.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>().clamp(-1.0, 1.0)
}

/// Cosine distance (1 - similarity) between two L2-normalised embeddings.
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    1.0 - cosine_similarity(a, b)
}

/// Verify that `embedding` is L2-normalised (norm within 1e-4 of 1.0).
pub fn is_normalised(embedding: &[f32]) -> bool {
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    (norm - 1.0).abs() < 1e-4
}
