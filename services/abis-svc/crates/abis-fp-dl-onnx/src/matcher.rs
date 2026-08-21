//! DL-based fingerprint matcher using cosine similarity.
//!
//! Reference: Engelsma et al., "Learning a Fixed-Length Fingerprint
//! Representation," IEEE TPAMI 2021.

use crate::embed::{DeepPrintEmbedder, Embedding};
use crate::{DlError, GrayImage, EMBED_DIM};

/// Cosine similarity between two unit-normalised embeddings.
/// Input embeddings must be L2-normalised; result is in [-1, 1] (clipped to [0, 1]).
pub fn cosine_similarity(a: &Embedding, b: &Embedding) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    // For unit vectors the dot product equals cosine similarity.
    // Clamp to [0, 1] because fingerprint genuine pairs should never be antipodal.
    dot.clamp(0.0, 1.0)
}

/// Deep-learning matcher: embeds two images and returns cosine similarity.
pub struct DlMatcher {
    embedder: DeepPrintEmbedder,
}

impl DlMatcher {
    pub fn new(embedder: DeepPrintEmbedder) -> Self {
        Self { embedder }
    }

    /// Compare two fingerprint images.  Returns cosine similarity in [0, 1].
    pub fn compare(&self, probe: &GrayImage, gallery: &GrayImage) -> Result<f32, DlError> {
        let e_probe = self.embedder.embed(probe)?;
        let e_gallery = self.embedder.embed(gallery)?;
        Ok(cosine_similarity(&e_probe, &e_gallery))
    }

    /// Compare pre-computed embeddings.
    pub fn compare_embeddings(a: &Embedding, b: &Embedding) -> f32 {
        cosine_similarity(a, b)
    }
}

impl Default for DlMatcher {
    fn default() -> Self {
        Self::new(DeepPrintEmbedder::from_env())
    }
}
