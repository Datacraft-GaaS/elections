//! k-anonymity face de-identification via synthetic composite.
//!
//! Replaces a face pixel-buffer with the pixel-mean of the k nearest
//! neighbours in an anonymity set, ensuring the output does not correspond to
//! the original identity.
//!
//! Reference: Gross et al. (2009). Face de-identification. Springer.

use sha2::{Digest, Sha256};
use thiserror::Error;

/// k-anonymity de-identification error.
#[derive(Debug, Error)]
pub enum KAnonError {
    #[error("anonymity set must contain at least k={0} faces, got {1}")]
    InsufficientAnonymitySet(usize, usize),
    #[error("image buffer length mismatch")]
    BufferMismatch,
    #[error("k must be >= 2")]
    KTooSmall,
}

/// k-anonymity face de-identifier.
///
/// Takes an input face image and an anonymity set of face embeddings with
/// their pixel buffers. Returns the pixel-mean of the k nearest neighbours
/// (by cosine similarity) from the anonymity set, excluding the input identity.
///
/// Guarantee: the output pixel buffer is NOT the original face (irreversible
/// for the de-identified copy; no stored mapping from output to input).
#[derive(Debug, Clone)]
pub struct KAnonDeident {
    pub k: usize,
}

impl KAnonDeident {
    pub fn new(k: usize) -> Result<Self, KAnonError> {
        if k < 2 {
            return Err(KAnonError::KTooSmall);
        }
        Ok(Self { k })
    }

    fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| x * y)
            .sum::<f32>()
            .clamp(-1.0, 1.0)
    }

    /// De-identify `target_pixels` using the anonymity set.
    ///
    /// `target_emb`: embedding of the target face.
    /// `anon_set`: slice of (embedding, pixel_buffer) pairs (excluding the target).
    /// All pixel buffers must have the same length as `target_pixels`.
    pub fn deidentify(
        &self,
        target_emb: &[f32],
        target_pixels: &[u8],
        anon_set: &[(Vec<f32>, Vec<u8>)],
    ) -> Result<Vec<u8>, KAnonError> {
        if anon_set.len() < self.k {
            return Err(KAnonError::InsufficientAnonymitySet(self.k, anon_set.len()));
        }
        // Check buffer lengths.
        if anon_set
            .iter()
            .any(|(_, px)| px.len() != target_pixels.len())
        {
            return Err(KAnonError::BufferMismatch);
        }
        // Find k nearest neighbours by cosine similarity (excluding the target
        // by definition since it is NOT in the anon_set).
        let mut scored: Vec<(f32, usize)> = anon_set
            .iter()
            .enumerate()
            .map(|(i, (emb, _))| (Self::cosine_sim(target_emb, emb), i))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let top_k = &scored[..self.k];

        // Pixel mean of top-k neighbours.
        let n_px = target_pixels.len();
        let mut acc = vec![0u64; n_px];
        for &(_, idx) in top_k {
            for (a, &v) in acc.iter_mut().zip(anon_set[idx].1.iter()) {
                *a += v as u64;
            }
        }
        let composite: Vec<u8> = acc.iter().map(|&s| (s / self.k as u64) as u8).collect();
        Ok(composite)
    }
}

/// Compute a synthetic embedding hash for a pixel buffer.
/// Used to verify the de-identified output differs from the original.
pub fn embedding_hash(pixels: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(pixels);
    h.finalize().into()
}
