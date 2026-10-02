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

//! Linear projection for cross-sensor iris template alignment.

use serde::{Deserialize, Serialize};

/// A learned linear projection: `output = weights * input + bias`.
///
/// `weights` is stored row-major as a flat `Vec<f32>` of shape
/// `[output_dim, input_dim]`; `bias` has length `output_dim`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearProjection {
    /// Flat row-major weight matrix of shape `[output_dim, input_dim]`.
    pub weights: Vec<f32>,
    /// Additive bias of length `output_dim`.
    pub bias: Vec<f32>,
    /// Number of input dimensions.
    pub input_dim: usize,
    /// Number of output dimensions.
    pub output_dim: usize,
}

impl LinearProjection {
    /// Construct an identity projection of the given dimension.
    ///
    /// The weight matrix is the identity and the bias is zero.
    pub fn identity(dim: usize) -> Self {
        let mut weights = vec![0.0f32; dim * dim];
        for i in 0..dim {
            weights[i * dim + i] = 1.0;
        }
        Self {
            weights,
            bias: vec![0.0; dim],
            input_dim: dim,
            output_dim: dim,
        }
    }

    /// Apply the projection: `output[i] = sum_j(weights[i,j] * code[j]) + bias[i]`.
    ///
    /// Returns a vector of length `output_dim`. If `code` is shorter than
    /// `input_dim` the missing values are treated as 0.
    pub fn apply(&self, code: &[f32]) -> Vec<f32> {
        (0..self.output_dim)
            .map(|i| {
                let row_start = i * self.input_dim;
                let dot: f32 = (0..self.input_dim)
                    .map(|j| {
                        let w = self.weights[row_start + j];
                        let x = code.get(j).copied().unwrap_or(0.0);
                        w * x
                    })
                    .sum();
                dot + self.bias[i]
            })
            .collect()
    }

    /// Fit a bias-correction projection from paired codes.
    ///
    /// Computes the mean difference `mean(gallery) - mean(probe)` per dimension
    /// and stores it as bias on top of an identity weight matrix. This is the
    /// simplest least-squares solution when only a bias shift is assumed.
    pub fn fit(probe_codes: &[Vec<f32>], gallery_codes: &[Vec<f32>]) -> Self {
        assert_eq!(
            probe_codes.len(),
            gallery_codes.len(),
            "paired codes must have equal length"
        );
        assert!(!probe_codes.is_empty(), "cannot fit on empty data");

        let dim = probe_codes[0].len();
        let n = probe_codes.len() as f32;

        let bias: Vec<f32> = (0..dim)
            .map(|d| {
                let sum_diff: f32 = probe_codes
                    .iter()
                    .zip(gallery_codes.iter())
                    .map(|(p, g)| {
                        g.get(d).copied().unwrap_or(0.0) - p.get(d).copied().unwrap_or(0.0)
                    })
                    .sum();
                sum_diff / n
            })
            .collect();

        let mut proj = Self::identity(dim);
        proj.bias = bias;
        proj
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_apply_returns_same_code() {
        let proj = LinearProjection::identity(4);
        let code = vec![1.0, 2.0, 3.0, 4.0];
        let out = proj.apply(&code);
        for (a, b) in out.iter().zip(code.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn fit_on_identical_pairs_returns_near_zero_bias() {
        let codes: Vec<Vec<f32>> = (0..10).map(|_| vec![0.5; 8]).collect();
        let proj = LinearProjection::fit(&codes, &codes);
        for b in &proj.bias {
            assert!(b.abs() < 1e-6);
        }
    }

    #[test]
    fn fit_produces_correct_dimensions() {
        let probe: Vec<Vec<f32>> = vec![vec![0.0; 16]; 5];
        let gallery: Vec<Vec<f32>> = vec![vec![1.0; 16]; 5];
        let proj = LinearProjection::fit(&probe, &gallery);
        assert_eq!(proj.input_dim, 16);
        assert_eq!(proj.output_dim, 16);
        assert_eq!(proj.bias.len(), 16);
    }
}
