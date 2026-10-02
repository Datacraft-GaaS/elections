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

//! CKKS cosine-similarity 1:1 match on encrypted face embeddings.
//!
//! Interface trait + toy implementation over 32-d integer polynomials.
//! Production swaps the toy impl for a real CKKS library via the trait.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CkksMatchError {
    #[error("ciphertext dimension mismatch: probe {probe} != gallery {gallery}")]
    DimensionMismatch { probe: usize, gallery: usize },
    #[error("empty ciphertext")]
    EmptyCiphertext,
    #[error("match score below threshold: {score:.4} < {threshold:.4}")]
    BelowThreshold { score: f64, threshold: f64 },
}

/// Toy CKKS ciphertext: stores plaintext values as integers.
/// Real implementation would hold polynomial ring elements.
#[derive(Debug, Clone)]
pub struct CkksCiphertext {
    pub values: Vec<i64>,
    pub scale: f64,
}

impl CkksCiphertext {
    /// Encrypts a float vector by scaling to integers.
    ///
    /// # Errors
    /// Returns an error if the slice is empty.
    pub fn encrypt(plain: &[f64], scale: f64) -> Result<Self, CkksMatchError> {
        if plain.is_empty() {
            return Err(CkksMatchError::EmptyCiphertext);
        }
        let values: Vec<i64> = plain.iter().map(|&v| (v * scale).round() as i64).collect();
        Ok(Self { values, scale })
    }

    /// Decrypts by reversing the scale.
    pub fn decrypt(&self) -> Vec<f64> {
        self.values.iter().map(|&v| v as f64 / self.scale).collect()
    }
}

/// Trait for CKKS cosine-similarity matching.
pub trait CkksMatchTrait {
    fn match_1v1(
        &self,
        probe: &CkksCiphertext,
        gallery: &CkksCiphertext,
        threshold: f64,
    ) -> Result<f64, CkksMatchError>;
}

/// Toy CKKS matcher: operates directly on scaled integers.
pub struct CkksMatcher {
    pub scale: f64,
}

impl CkksMatcher {
    pub fn new(scale: f64) -> Self {
        Self { scale }
    }
}

impl CkksMatchTrait for CkksMatcher {
    /// Computes homomorphic cosine similarity.
    ///
    /// # Errors
    /// Returns errors on dimension mismatch or below-threshold score.
    fn match_1v1(
        &self,
        probe: &CkksCiphertext,
        gallery: &CkksCiphertext,
        threshold: f64,
    ) -> Result<f64, CkksMatchError> {
        if probe.values.is_empty() || gallery.values.is_empty() {
            return Err(CkksMatchError::EmptyCiphertext);
        }
        if probe.values.len() != gallery.values.len() {
            return Err(CkksMatchError::DimensionMismatch {
                probe: probe.values.len(),
                gallery: gallery.values.len(),
            });
        }

        // Toy HE: compute over plaintext (correct semantics for small parameters).
        let dot: i64 = probe
            .values
            .iter()
            .zip(gallery.values.iter())
            .map(|(a, b)| a * b)
            .sum();
        let norm_a: f64 = probe
            .values
            .iter()
            .map(|&v| (v * v) as f64)
            .sum::<f64>()
            .sqrt();
        let norm_b: f64 = gallery
            .values
            .iter()
            .map(|&v| (v * v) as f64)
            .sum::<f64>()
            .sqrt();

        let score = if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            (dot as f64 / (norm_a * norm_b)).clamp(-1.0, 1.0)
        };

        if score < threshold {
            return Err(CkksMatchError::BelowThreshold { score, threshold });
        }
        Ok(score)
    }
}
