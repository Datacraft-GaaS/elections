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

//! User-specific random projection (Johnson-Lindenstrauss).
//!
//! Reduces embedding dimension while preserving pairwise distances up to eps.

use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectionError {
    #[error("input dimension must be > 0")]
    ZeroDimension,
    #[error("output dimension must be <= input dimension")]
    InvalidOutputDim,
    #[error("user key must not be empty")]
    EmptyKey,
    #[error("input length {actual} != expected {expected}")]
    DimensionMismatch { actual: usize, expected: usize },
}

/// Derives a random projection matrix from a user key and projects vectors.
pub struct RandomProjector {
    input_dim: usize,
    output_dim: usize,
    /// Flattened projection matrix: output_dim x input_dim entries in [-1, +1].
    matrix: Vec<f64>,
}

impl RandomProjector {
    /// Constructs a random projector for the given user key.
    ///
    /// The projection matrix is pseudo-randomly derived from `user_key` so that
    /// it is unique per user, making projected templates unlinkable across schemes.
    ///
    /// # Errors
    /// Returns an error if dimensions are invalid or the key is empty.
    pub fn new(
        input_dim: usize,
        output_dim: usize,
        user_key: &[u8],
    ) -> Result<Self, ProjectionError> {
        if input_dim == 0 {
            return Err(ProjectionError::ZeroDimension);
        }
        if output_dim == 0 || output_dim > input_dim {
            return Err(ProjectionError::InvalidOutputDim);
        }
        if user_key.is_empty() {
            return Err(ProjectionError::EmptyKey);
        }

        let total = output_dim * input_dim;
        let mut matrix = Vec::with_capacity(total);
        let mut counter = 0u64;
        while matrix.len() < total {
            let mut h = Sha256::new();
            h.update(user_key);
            h.update(counter.to_le_bytes());
            let digest = h.finalize();
            for byte in digest.iter() {
                // Map byte range [0,255] -> [-1.0, +1.0].
                matrix.push((*byte as f64 / 127.5) - 1.0);
                if matrix.len() == total {
                    break;
                }
            }
            counter += 1;
        }

        Ok(Self {
            input_dim,
            output_dim,
            matrix,
        })
    }

    /// Projects a feature vector to the lower-dimensional space.
    ///
    /// # Errors
    /// Returns `ProjectionError::DimensionMismatch` if the input length is wrong.
    pub fn project(&self, feature: &[f64]) -> Result<Vec<f64>, ProjectionError> {
        if feature.len() != self.input_dim {
            return Err(ProjectionError::DimensionMismatch {
                actual: feature.len(),
                expected: self.input_dim,
            });
        }

        let scale = 1.0 / (self.output_dim as f64).sqrt();
        let mut output = vec![0.0f64; self.output_dim];
        for (out_idx, row) in self.matrix.chunks_exact(self.input_dim).enumerate() {
            output[out_idx] = row
                .iter()
                .zip(feature.iter())
                .map(|(m, x)| m * x)
                .sum::<f64>()
                * scale;
        }
        Ok(output)
    }

    pub fn input_dim(&self) -> usize {
        self.input_dim
    }
    pub fn output_dim(&self) -> usize {
        self.output_dim
    }
}

/// Euclidean distance between two vectors of equal length.
pub fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}
