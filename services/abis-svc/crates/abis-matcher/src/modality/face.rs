//! Face template: 512-d L2-normalised embedding + ICAO conformance.

use super::{Modality, Template};

/// ICAO 9303 conformance flags.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IcaoFlags {
    pub pose_ok: bool,
    pub lighting_ok: bool,
    pub eyes_open: bool,
    pub no_glasses: bool,
}

impl IcaoFlags {
    /// Aggregate ICAO quality score in [0.0, 1.0].
    pub fn score(&self) -> f32 {
        let flags = [
            self.pose_ok,
            self.lighting_ok,
            self.eyes_open,
            self.no_glasses,
        ];
        flags.iter().filter(|&&f| f).count() as f32 / flags.len() as f32
    }

    pub fn all_pass(&self) -> bool {
        self.pose_ok && self.lighting_ok && self.eyes_open && self.no_glasses
    }
}

/// Eye-corner and pose landmarks.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FaceLandmarks {
    /// Left eye corner (x, y) in normalised image coords [0, 1].
    pub left_eye: (f32, f32),
    /// Right eye corner (x, y).
    pub right_eye: (f32, f32),
    /// Head roll in degrees.
    pub roll: f32,
    /// Head yaw in degrees.
    pub yaw: f32,
    /// Head pitch in degrees.
    pub pitch: f32,
}

/// 512-dimensional face embedding template.
///
/// The embedding must be L2-normalised; `normalise()` enforces this.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FaceTemplate {
    /// L2-normalised 512-d float32 embedding.
    pub embedding: Vec<f32>,
    pub landmarks: Option<FaceLandmarks>,
    pub icao: IcaoFlags,
    /// ICAO face quality score [0.0, 1.0] (ISO 29794-5).
    pub quality_score: f32,
}

impl FaceTemplate {
    /// Construct from a raw embedding vector (normalises automatically).
    pub fn new(embedding: Vec<f32>, icao: IcaoFlags, landmarks: Option<FaceLandmarks>) -> Self {
        assert_eq!(embedding.len(), 512, "embedding must be 512-d");
        let quality_score = icao.score();
        let mut t = Self {
            embedding,
            landmarks,
            icao,
            quality_score,
        };
        t = t.normalise();
        t
    }

    /// Cosine distance to another template (both assumed L2-normalised).
    pub fn cosine_distance(&self, other: &Self) -> f32 {
        1.0 - self.dot_product(other)
    }

    /// Dot product (= cosine similarity for L2-normalised vectors).
    pub fn dot_product(&self, other: &Self) -> f32 {
        self.embedding
            .iter()
            .zip(other.embedding.iter())
            .map(|(a, b)| a * b)
            .sum()
    }

    /// L2 norm of the embedding.
    pub fn l2_norm(&self) -> f32 {
        self.embedding.iter().map(|x| x * x).sum::<f32>().sqrt()
    }
}

impl Template for FaceTemplate {
    fn modality() -> Modality {
        Modality::Face
    }

    fn quality(&self) -> f32 {
        self.quality_score.clamp(0.0, 1.0)
    }

    fn feature_bytes(&self) -> Vec<u8> {
        // Little-endian f32 bytes.
        self.embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect()
    }

    fn normalise(&self) -> Self {
        let norm = self.l2_norm();
        if norm < 1e-9 {
            return self.clone();
        }
        let embedding = self.embedding.iter().map(|x| x / norm).collect();
        Self {
            embedding,
            ..self.clone()
        }
    }
}
