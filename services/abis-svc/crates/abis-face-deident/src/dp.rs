//! Differential-privacy Gaussian noise on face embeddings for publication.
//!
//! Reference: Dwork, C. & Roth, A. (2014). "The Algorithmic Foundations of
//! Differential Privacy." FnTCS 9(3-4). Chapter 3.5: Gaussian mechanism.
//!
//! Gaussian mechanism: add N(0, sigma^2) noise to each dimension, where
//! sigma = sensitivity * sqrt(2 * ln(1.25/delta)) / epsilon.
//! Sensitivity of an L2-normalised embedding = 2 (max L2 distance).

use sha2::{Digest, Sha256};
use thiserror::Error;

/// DP noise error.
#[derive(Debug, Error)]
pub enum DpError {
    #[error("epsilon must be positive")]
    InvalidEpsilon,
    #[error("delta must be in (0, 1)")]
    InvalidDelta,
    #[error("embedding must be non-empty")]
    EmptyEmbedding,
}

/// Differential-privacy Gaussian noise adder for face embeddings.
///
/// Epsilon and delta are the DP privacy parameters. The output embedding
/// is L2-renormalised after adding noise.
#[derive(Debug, Clone)]
pub struct DpEmbeddingNoise {
    /// Privacy loss parameter (smaller = more private).
    pub epsilon: f64,
    /// Delta (probability of epsilon-privacy failure).
    pub delta: f64,
}

impl Default for DpEmbeddingNoise {
    /// Default: epsilon=1.0, delta=1e-5 (reasonable for publication).
    fn default() -> Self {
        Self {
            epsilon: 1.0,
            delta: 1e-5,
        }
    }
}

impl DpEmbeddingNoise {
    pub fn new(epsilon: f64, delta: f64) -> Result<Self, DpError> {
        if epsilon <= 0.0 {
            return Err(DpError::InvalidEpsilon);
        }
        if !(0.0 < delta && delta < 1.0) {
            return Err(DpError::InvalidDelta);
        }
        Ok(Self { epsilon, delta })
    }

    /// Gaussian sigma for the Gaussian mechanism.
    /// sigma = sensitivity * sqrt(2 * ln(1.25/delta)) / epsilon.
    /// L2 sensitivity of a normalised embedding is 2.
    pub fn sigma(&self) -> f64 {
        let sensitivity = 2.0f64;
        sensitivity * (2.0 * (1.25 / self.delta).ln()).sqrt() / self.epsilon
    }

    /// Add calibrated Gaussian noise to `embedding` using a deterministic
    /// hash-based PRNG seeded with `seed` (for reproducibility in tests).
    ///
    /// In production use a cryptographically secure RNG; this stub uses
    /// a SHA-256 counter-based construction for CI determinism.
    pub fn add_noise(&self, embedding: &[f32], seed: u64) -> Result<Vec<f32>, DpError> {
        if embedding.is_empty() {
            return Err(DpError::EmptyEmbedding);
        }
        let sigma = self.sigma() as f32;
        let mut out = Vec::with_capacity(embedding.len());

        for (i, &v) in embedding.iter().enumerate() {
            // Box-Muller transform via hash-based uniform samples.
            // We need two independent uniform samples per dimension.
            let u1 = hash_uniform(seed, (i * 2) as u64);
            let u2 = hash_uniform(seed, (i * 2 + 1) as u64);
            // Box-Muller: N(0,1)
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
            out.push(v + sigma * z);
        }

        // L2-renormalise.
        let norm: f32 = out.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
        out.iter_mut().for_each(|x| *x /= norm);
        Ok(out)
    }
}

/// Hash a (seed, index) pair to a uniform f32 in (0, 1).
fn hash_uniform(seed: u64, index: u64) -> f32 {
    let mut h = Sha256::new();
    h.update(seed.to_le_bytes());
    h.update(index.to_le_bytes());
    let d: [u8; 32] = h.finalize().into();
    let raw = u32::from_le_bytes([d[0], d[1], d[2], d[3]]) as f32;
    // Map to (0, 1) excluding exact 0 to avoid ln(0).
    (raw / u32::MAX as f32).clamp(1e-9, 1.0 - 1e-9)
}
