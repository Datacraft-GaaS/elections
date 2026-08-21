//! Deterministic hash-based face embedder for CI without ONNX model files.
//!
//! Properties:
//! - Identical input bytes => identical L2-normalised embedding.
//! - Different inputs => embeddings with cosine similarity < 0.4 (with high
//!   probability, by birthday-paradox argument on the 512-d hash spread).
//!
//! Implementation: SHA-256 of successive 16-byte seeds derived from the input
//! hash. Each 4-byte chunk of SHA-256 output yields one f32 via direct
//! interpretation, producing 8 floats per hash call. We need 512 floats so we
//! call SHA-256 64 times with different salt values.
//!
//! Reference: This hash-based stub follows the pattern used in NIST SP
//! 800-90A (DRBG) for seeding, adapted for unit-test embedding surrogates.

use crate::{l2_normalise, EmbedError, FaceEmbedder, EMBED_DIM};
use sha2::{Digest, Sha256};

/// Deterministic hash-based face embedder.
///
/// Produces a valid 512-d L2-normalised embedding from any input without
/// requiring an ONNX model file. Suitable for CI and integration tests.
#[derive(Debug, Clone, Default)]
pub struct HashEmbedder;

impl HashEmbedder {
    pub fn new() -> Self {
        Self
    }

    /// Derive a 512-d raw float array from raw bytes using iterated SHA-256.
    fn derive_raw(input: &[u8]) -> [f32; EMBED_DIM] {
        // Step 1: hash the input to a 32-byte seed.
        let seed: [u8; 32] = {
            let mut h = Sha256::new();
            h.update(input);
            h.finalize().into()
        };

        // Step 2: expand to 512 floats using 64 iterated rounds.
        // Round i hashes (seed || i_as_u8) producing 8 f32s per round.
        let mut result = [0f32; EMBED_DIM];
        for i in 0u8..64 {
            let mut h = Sha256::new();
            h.update(seed);
            h.update([i]);
            let digest: [u8; 32] = h.finalize().into();
            for (j, chunk) in digest.chunks(4).enumerate() {
                let idx = (i as usize) * 8 + j;
                if idx < EMBED_DIM {
                    let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
                    // Map 4 bytes to a signed float in [-1, 1].
                    let raw = i32::from_le_bytes(bytes) as f32 / i32::MAX as f32;
                    result[idx] = raw;
                }
            }
        }
        result
    }
}

impl FaceEmbedder for HashEmbedder {
    fn embed(&self, aligned_rgb: &[u8]) -> Result<[f32; EMBED_DIM], EmbedError> {
        if aligned_rgb.is_empty() {
            return Err(EmbedError::InvalidInput);
        }
        let mut v = Self::derive_raw(aligned_rgb);
        l2_normalise(&mut v);
        Ok(v)
    }
}
