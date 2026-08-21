//! Voice matching via cosine similarity on embeddings and DTW on MFCC frames.

use serde::{Deserialize, Serialize};

use crate::domain::VoiceTemplate;

/// Scoring method used for the match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchMethod {
    Cosine,
    Dtw,
}

/// Match score for a voice comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub raw: f32,
    pub threshold: f32,
    pub threshold_pass: bool,
    pub method: MatchMethod,
}

/// Default threshold for cosine-based voice matching.
pub const DEFAULT_THRESHOLD_COSINE: f32 = 0.75;
/// Default threshold for DTW-based matching (lower distance = better).
pub const DEFAULT_THRESHOLD_DTW: f32 = 0.60;

/// Trait for plugging in external embedding extractors (e.g. ECAPA-TDNN).
pub trait VendorAdapter: Send + Sync {
    fn extract_embedding(&self, mfcc: &[Vec<f32>]) -> Vec<f32>;
}

/// Compute cosine similarity between two embedding vectors.
///
/// Returns 0.0 for zero-norm inputs.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a < 1e-10 || norm_b < 1e-10 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

/// Compute Dynamic Time Warping distance between two MFCC frame sequences.
///
/// Returns 0.0 for identical sequences; larger values indicate more dissimilarity.
/// Normalized by path length to be sequence-length invariant.
pub fn dtw_distance(probe_frames: &[Vec<f32>], gallery_frames: &[Vec<f32>]) -> f32 {
    let m = probe_frames.len();
    let n = gallery_frames.len();

    if m == 0 && n == 0 {
        return 0.0;
    }
    if m == 0 || n == 0 {
        return f32::INFINITY;
    }

    let frame_dist = |i: usize, j: usize| -> f32 {
        probe_frames[i]
            .iter()
            .zip(gallery_frames[j].iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f32>()
            .sqrt()
    };

    let mut dtw = vec![vec![f32::INFINITY; n]; m];
    dtw[0][0] = frame_dist(0, 0);

    for i in 1..m {
        dtw[i][0] = dtw[i - 1][0] + frame_dist(i, 0);
    }
    for j in 1..n {
        dtw[0][j] = dtw[0][j - 1] + frame_dist(0, j);
    }

    for i in 1..m {
        for j in 1..n {
            let min_prev = dtw[i - 1][j].min(dtw[i][j - 1]).min(dtw[i - 1][j - 1]);
            dtw[i][j] = min_prev + frame_dist(i, j);
        }
    }

    // Normalize by path length.
    dtw[m - 1][n - 1] / (m + n) as f32
}

/// Match two voice templates using cosine similarity on their embeddings.
pub fn match_voice_cosine(probe: &VoiceTemplate, gallery: &VoiceTemplate) -> Score {
    let raw = cosine_similarity(&probe.embedding, &gallery.embedding);
    Score {
        raw,
        threshold: DEFAULT_THRESHOLD_COSINE,
        threshold_pass: raw >= DEFAULT_THRESHOLD_COSINE,
        method: MatchMethod::Cosine,
    }
}

/// Match two voice templates using DTW on MFCC feature frames.
///
/// DTW distance is converted to a similarity score in [0, 1].
pub fn match_voice_dtw(probe: &VoiceTemplate, gallery: &VoiceTemplate) -> Score {
    let dist = dtw_distance(&probe.mfcc_features, &gallery.mfcc_features);
    // Convert distance to similarity: higher distance = lower score.
    let raw = if dist.is_infinite() {
        0.0
    } else {
        1.0 / (1.0 + dist)
    };
    Score {
        raw,
        threshold: DEFAULT_THRESHOLD_DTW,
        threshold_pass: raw >= DEFAULT_THRESHOLD_DTW,
        method: MatchMethod::Dtw,
    }
}
