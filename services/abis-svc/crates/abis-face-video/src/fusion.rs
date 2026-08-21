//! Quality-weighted embedding fusion across a track.
//!
//! Reference: Best & Best (2021). "Quality-weighted averaging of face
//! embeddings across video frames." FG 2021.

use crate::track::Track;

/// Fused face template derived from a multi-frame track.
#[derive(Debug, Clone)]
pub struct FusedTemplate {
    pub track_id: u64,
    /// Quality-weighted mean embedding (L2-normalised), or None if no
    /// frame had an embedding.
    pub embedding: Option<Vec<f32>>,
    /// Sum of quality weights used.
    pub total_weight: f32,
    /// Number of frames that contributed an embedding.
    pub frame_count: usize,
}

/// Fuse embeddings from a track using quality-weighted averaging.
///
/// Frames with quality = 0.0 contribute zero weight and are effectively
/// excluded. The resulting vector is L2-normalised.
pub fn fuse_track(track: &Track) -> FusedTemplate {
    let mut acc: Vec<f64> = Vec::new();
    let mut total_weight = 0.0f64;
    let mut frame_count = 0usize;

    for det in &track.detections {
        let Some(emb) = &det.embedding else { continue };
        let w = det.quality.max(0.0) as f64;
        if w < 1e-9 {
            continue;
        }

        if acc.is_empty() {
            acc = vec![0.0f64; emb.len()];
        }
        if emb.len() != acc.len() {
            continue;
        }

        for (a, &e) in acc.iter_mut().zip(emb.iter()) {
            *a += w * e as f64;
        }
        total_weight += w;
        frame_count += 1;
    }

    if total_weight < 1e-9 || acc.is_empty() {
        return FusedTemplate {
            track_id: track.id,
            embedding: None,
            total_weight: 0.0,
            frame_count: 0,
        };
    }

    // Normalise the weighted sum.
    let mut v: Vec<f32> = acc.iter().map(|&x| (x / total_weight) as f32).collect();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-9 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }

    FusedTemplate {
        track_id: track.id,
        embedding: Some(v),
        total_weight: total_weight as f32,
        frame_count,
    }
}
