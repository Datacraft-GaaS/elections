//! Age-invariant face matching.
//!
//! References:
//! - Guo, G. et al. (2019). "A Survey on Age Estimation and Face Aging."
//!   ACM Computing Surveys 52(6).
//! - Wang, Z. et al. (2022). "Age-Invariant Face Recognition via Partial
//!   Factor Pooling." CVPR 2022. (residual age-normalisation approach)
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const EMBED_DIM: usize = 512;

/// Age estimation error.
#[derive(Debug, Error)]
pub enum AgeError {
    #[error("input embedding is empty")]
    EmptyInput,
    #[error("embedding dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
}

/// Age estimate with confidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeEstimate {
    /// Estimated integer age in [0, 100].
    pub age: u8,
    /// Confidence in [0.0, 1.0].
    pub confidence: f32,
}

/// Age estimator interface.
pub trait AgeEstimator {
    fn estimate(&self, embedding: &[f32]) -> Result<AgeEstimate, AgeError>;
}

/// Hash-based deterministic age estimator for CI (no model files required).
///
/// Maps the embedding hash to a plausible age in [15, 80] with mid confidence.
#[derive(Debug, Clone, Default)]
pub struct HashAgeEstimator;

impl HashAgeEstimator {
    pub fn new() -> Self {
        Self
    }
}

impl AgeEstimator for HashAgeEstimator {
    fn estimate(&self, embedding: &[f32]) -> Result<AgeEstimate, AgeError> {
        if embedding.is_empty() {
            return Err(AgeError::EmptyInput);
        }
        let bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        let hash: [u8; 32] = {
            let mut h = Sha256::new();
            h.update(&bytes);
            h.finalize().into()
        };
        let age_raw = hash[0] as u16 + hash[1] as u16;
        let age = (15 + (age_raw % 65)) as u8; // [15, 79]
        let conf = 0.6 + (hash[2] as f32 / 255.0) * 0.3;
        Ok(AgeEstimate {
            age,
            confidence: conf.clamp(0.0, 1.0),
        })
    }
}

/// Age-invariant matcher: subtracts a residual age-direction vector from both
/// embeddings before computing cosine similarity.
///
/// The age-direction vector is estimated empirically as the mean difference
/// between old and young faces; here it is a deterministic stub.
///
/// Reference: Wang et al. (2022). CVPR 2022.
#[derive(Debug, Clone)]
pub struct AgeInvariantMatcher {
    /// Age-direction residual vector (512-d, L2-normalised).
    age_direction: Vec<f32>,
    /// Strength of age-direction removal (0 = off, 1 = full removal).
    lambda: f32,
}

impl Default for AgeInvariantMatcher {
    fn default() -> Self {
        // Stub age direction: unit vector along dimension 0.
        let mut v = vec![0.0f32; EMBED_DIM];
        v[0] = 1.0;
        Self {
            age_direction: v,
            lambda: 0.3,
        }
    }
}

impl AgeInvariantMatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_lambda(mut self, lambda: f32) -> Self {
        self.lambda = lambda.clamp(0.0, 1.0);
        self
    }

    /// Project out the age component from an embedding.
    fn deage(&self, emb: &[f32]) -> Vec<f32> {
        let dot: f32 = emb
            .iter()
            .zip(self.age_direction.iter())
            .map(|(a, b)| a * b)
            .sum();
        let mut out: Vec<f32> = emb
            .iter()
            .zip(self.age_direction.iter())
            .map(|(e, d)| e - self.lambda * dot * d)
            .collect();
        let norm: f32 = out.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-9 {
            for x in out.iter_mut() {
                *x /= norm;
            }
        }
        out
    }

    /// Cosine similarity after removing the age component.
    pub fn similarity(&self, a: &[f32], b: &[f32]) -> Result<f32, AgeError> {
        if a.len() != EMBED_DIM {
            return Err(AgeError::DimensionMismatch {
                expected: EMBED_DIM,
                got: a.len(),
            });
        }
        if b.len() != EMBED_DIM {
            return Err(AgeError::DimensionMismatch {
                expected: EMBED_DIM,
                got: b.len(),
            });
        }
        let da = self.deage(a);
        let db = self.deage(b);
        let sim: f32 = da.iter().zip(db.iter()).map(|(x, y)| x * y).sum();
        Ok(sim.clamp(-1.0, 1.0))
    }
}

/// Age-progression stub: simulates appearance after N years by adding a
/// small age-direction perturbation to the embedding.
///
/// This is a numerical stub; real progression requires a generative model.
#[derive(Debug, Clone, Default)]
pub struct AgeProgressionStub;

impl AgeProgressionStub {
    pub fn new() -> Self {
        Self
    }

    /// Simulate age progression by `years` (5/10/15 supported).
    /// Returns a perturbed L2-normalised embedding.
    pub fn progress(&self, embedding: &[f32], years: u32) -> Result<Vec<f32>, AgeError> {
        if embedding.len() != EMBED_DIM {
            return Err(AgeError::DimensionMismatch {
                expected: EMBED_DIM,
                got: embedding.len(),
            });
        }
        let alpha = (years as f32 / 100.0).clamp(0.0, 0.5);
        // Add a small perturbation to simulate aging drift.
        let mut out: Vec<f32> = embedding
            .iter()
            .enumerate()
            .map(|(i, &v)| v + alpha * if i % 3 == 0 { 0.01 } else { -0.005 })
            .collect();
        let norm: f32 = out.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-9 {
            for x in out.iter_mut() {
                *x /= norm;
            }
        }
        Ok(out)
    }
}
