//! Iris embedding interface and cosine similarity utilities.

use eel_abis_iris_segment::GrayImage;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::hash_embedder::HashEmbedder;

/// A dense floating-point embedding vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub vector: Vec<f32>,
}

impl Embedding {
    /// Euclidean norm (L2 norm) of the vector.
    pub fn l2_norm(&self) -> f32 {
        self.vector.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// Cosine similarity with another embedding.
    ///
    /// # Errors
    /// Returns `EmbedError::DimensionMismatch` when vectors differ in length.
    /// Returns `EmbedError::ZeroVector` when either vector has zero norm.
    pub fn cosine_similarity(&self, other: &Self) -> Result<f32, EmbedError> {
        if self.vector.len() != other.vector.len() {
            return Err(EmbedError::DimensionMismatch {
                a: self.vector.len(),
                b: other.vector.len(),
            });
        }

        let norm_a = self.l2_norm();
        let norm_b = other.l2_norm();
        if norm_a < f32::EPSILON || norm_b < f32::EPSILON {
            return Err(EmbedError::ZeroVector);
        }

        let dot: f32 = self
            .vector
            .iter()
            .zip(other.vector.iter())
            .map(|(a, b)| a * b)
            .sum();
        Ok((dot / (norm_a * norm_b)).clamp(-1.0, 1.0))
    }

    /// Returns true when the vector is unit-normalised (|norm - 1.0| < 0.01).
    pub fn is_unit(&self) -> bool {
        (self.l2_norm() - 1.0).abs() < 0.01
    }
}

/// Errors that can occur during embedding.
#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("dimension mismatch: a={a}, b={b}")]
    DimensionMismatch { a: usize, b: usize },
    #[error("zero vector: image may be all-zero or featureless")]
    ZeroVector,
}

/// Iris image embedder. Produces unit-normalised 256-dimensional vectors.
pub struct IrisEmbedder;

impl IrisEmbedder {
    /// Embed a grayscale iris image into a unit-normalised 256-d vector.
    ///
    /// # Errors
    /// Returns `EmbedError::ZeroVector` when the image contains no signal.
    pub fn embed(image: &GrayImage) -> Result<Embedding, EmbedError> {
        if image.pixels().iter().all(|&p| p == 0) {
            return Err(EmbedError::ZeroVector);
        }

        let embedding = HashEmbedder::embed(image);

        if !embedding.is_unit() {
            tracing::warn!(
                norm = embedding.l2_norm(),
                "embedding is not unit-normalised"
            );
        }

        Ok(embedding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_embedding(dim: usize) -> Embedding {
        let val = 1.0 / (dim as f32).sqrt();
        Embedding {
            vector: vec![val; dim],
        }
    }

    fn make_image(w: u32, h: u32, fill: u8) -> GrayImage {
        GrayImage::from_raw(w, h, vec![fill; (w * h) as usize])
    }

    #[test]
    fn l2_norm_of_unit_vector_is_one() {
        let e = unit_embedding(256);
        assert!((e.l2_norm() - 1.0).abs() < 1e-5, "norm: {}", e.l2_norm());
    }

    #[test]
    fn cosine_similarity_identical_is_one() {
        let e = unit_embedding(256);
        let sim = e.cosine_similarity(&e).expect("should compute");
        assert!((sim - 1.0).abs() < 1e-5, "sim: {sim}");
    }

    #[test]
    fn cosine_similarity_different_less_than_one() {
        let e1 = Embedding {
            vector: vec![1.0, 0.0, 0.0],
        };
        let e2 = Embedding {
            vector: vec![0.0, 1.0, 0.0],
        };
        let sim = e1.cosine_similarity(&e2).expect("should compute");
        assert!(sim < 1.0, "sim should be < 1.0: {sim}");
    }

    #[test]
    fn cosine_similarity_dimension_mismatch_errors() {
        let e1 = Embedding {
            vector: vec![1.0; 64],
        };
        let e2 = Embedding {
            vector: vec![1.0; 128],
        };
        assert!(matches!(
            e1.cosine_similarity(&e2),
            Err(EmbedError::DimensionMismatch { .. })
        ));
    }

    #[test]
    fn cosine_similarity_in_range() {
        let e1 = Embedding {
            vector: vec![0.5, 0.5, 0.5, 0.5],
        };
        let e2 = Embedding {
            vector: vec![0.1, 0.9, 0.3, 0.7],
        };
        let sim = e1.cosine_similarity(&e2).expect("should compute");
        assert!(sim >= -1.0 && sim <= 1.0, "sim out of range: {sim}");
    }

    #[test]
    fn is_unit_true_for_unit_embedding() {
        let e = unit_embedding(256);
        assert!(e.is_unit());
    }

    #[test]
    fn embed_returns_unit_vector() {
        let img = make_image(32, 32, 128);
        let e = IrisEmbedder::embed(&img).expect("should embed");
        assert!(e.is_unit(), "norm: {}", e.l2_norm());
    }

    #[test]
    fn embed_produces_256d_vector() {
        let img = make_image(32, 32, 64);
        let e = IrisEmbedder::embed(&img).expect("should embed");
        assert_eq!(e.vector.len(), 256);
    }

    #[test]
    fn identical_images_produce_identical_embeddings() {
        let img1 = make_image(16, 16, 200);
        let img2 = make_image(16, 16, 200);
        let e1 = IrisEmbedder::embed(&img1).expect("embed1");
        let e2 = IrisEmbedder::embed(&img2).expect("embed2");
        assert_eq!(e1.vector, e2.vector);
    }

    #[test]
    fn different_images_produce_different_embeddings() {
        let img1 = make_image(16, 16, 100);
        let img2 = make_image(16, 16, 200);
        let e1 = IrisEmbedder::embed(&img1).expect("embed1");
        let e2 = IrisEmbedder::embed(&img2).expect("embed2");
        assert_ne!(e1.vector, e2.vector);
    }

    #[test]
    fn all_zero_image_returns_zero_vector_error() {
        let img = make_image(16, 16, 0);
        assert!(matches!(
            IrisEmbedder::embed(&img),
            Err(EmbedError::ZeroVector)
        ));
    }

    #[test]
    fn embedding_serde_round_trip() {
        let e = unit_embedding(256);
        let json = serde_json::to_string(&e).expect("serialise");
        let back: Embedding = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(e.vector.len(), back.vector.len());
        for (a, b) in e.vector.iter().zip(back.vector.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn is_unit_true_for_embed_result() {
        let img = make_image(32, 32, 77);
        let e = IrisEmbedder::embed(&img).expect("embed");
        assert!(e.is_unit());
    }
}
