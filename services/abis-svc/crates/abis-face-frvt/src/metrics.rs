//! FRVT metric computation.
//!
//! Reference: NIST FRVT 1:1 and 1:N protocols (frvt.nist.gov).

use crate::corpus::{FrvtCorpus, FrvtImage};
use serde::{Deserialize, Serialize};

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x * y)
        .sum::<f32>()
        .clamp(-1.0, 1.0)
}

/// 1:1 verification error rates at a given threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FmrFnmr {
    pub threshold: f32,
    /// False Match Rate: fraction of impostor pairs accepted.
    pub fmr: f32,
    /// False Non-Match Rate: fraction of genuine pairs rejected.
    pub fnmr: f32,
    pub genuine_count: usize,
    pub impostor_count: usize,
}

impl FmrFnmr {
    pub fn compute(corpus: &FrvtCorpus, threshold: f32) -> Self {
        let genuines = corpus.genuine_pairs();
        let impostors = corpus.impostor_pairs();

        let genuine_accept: usize = genuines
            .iter()
            .filter(|(a, b)| cosine_sim(&a.embedding, &b.embedding) >= threshold)
            .count();
        let impostor_accept: usize = impostors
            .iter()
            .filter(|(a, b)| cosine_sim(&a.embedding, &b.embedding) >= threshold)
            .count();

        let fnmr = if genuines.is_empty() {
            0.0
        } else {
            1.0 - genuine_accept as f32 / genuines.len() as f32
        };
        let fmr = if impostors.is_empty() {
            0.0
        } else {
            impostor_accept as f32 / impostors.len() as f32
        };

        Self {
            threshold,
            fmr: fmr.clamp(0.0, 1.0),
            fnmr: fnmr.clamp(0.0, 1.0),
            genuine_count: genuines.len(),
            impostor_count: impostors.len(),
        }
    }
}

/// 1:N identification: Rank-N hit rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankN {
    pub rank: usize,
    /// Fraction of probes where the true identity appears in the top-N results.
    pub hit_rate: f32,
    pub probe_count: usize,
}

impl RankN {
    pub fn compute(corpus: &FrvtCorpus, gallery_set: &str, probe_set: &str, rank: usize) -> Self {
        // Separate gallery and probe by image_set name (naive split by first char of id).
        let gallery: Vec<&FrvtImage> = corpus
            .images
            .iter()
            .filter(|img| img.id.starts_with(gallery_set))
            .collect();
        let probes: Vec<&FrvtImage> = corpus
            .images
            .iter()
            .filter(|img| img.id.starts_with(probe_set))
            .collect();

        if probes.is_empty() || gallery.is_empty() {
            return Self {
                rank,
                hit_rate: 0.0,
                probe_count: 0,
            };
        }

        let hits: usize = probes
            .iter()
            .filter(|probe| {
                let mut scores: Vec<(f32, &str)> = gallery
                    .iter()
                    .map(|g| {
                        (
                            cosine_sim(&probe.embedding, &g.embedding),
                            g.subject_id.as_str(),
                        )
                    })
                    .collect();
                scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
                let top_n_subjects: Vec<&str> = scores.iter().take(rank).map(|(_, s)| *s).collect();
                top_n_subjects.contains(&probe.subject_id.as_str())
            })
            .count();

        Self {
            rank,
            hit_rate: (hits as f32 / probes.len() as f32).clamp(0.0, 1.0),
            probe_count: probes.len(),
        }
    }
}

/// Quality-FNMR correlation: does higher quality predict lower FNMR?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCorrelation {
    /// Pearson correlation between quality score and match success (1=genuine match).
    pub pearson_r: f32,
}

impl QualityCorrelation {
    pub fn compute(corpus: &FrvtCorpus, threshold: f32) -> Self {
        let pairs = corpus.genuine_pairs();
        if pairs.len() < 2 {
            return Self { pearson_r: 0.0 };
        }
        // For each genuine pair use min(quality_a, quality_b) and match success.
        let xs: Vec<f32> = pairs
            .iter()
            .map(|(a, b)| a.quality.min(b.quality))
            .collect();
        let ys: Vec<f32> = pairs
            .iter()
            .map(|(a, b)| {
                if cosine_sim(&a.embedding, &b.embedding) >= threshold {
                    1.0
                } else {
                    0.0
                }
            })
            .collect();
        let n = xs.len() as f32;
        let mu_x: f32 = xs.iter().sum::<f32>() / n;
        let mu_y: f32 = ys.iter().sum::<f32>() / n;
        let cov: f32 = xs
            .iter()
            .zip(ys.iter())
            .map(|(x, y)| (x - mu_x) * (y - mu_y))
            .sum::<f32>()
            / n;
        let std_x: f32 = (xs.iter().map(|x| (x - mu_x).powi(2)).sum::<f32>() / n).sqrt();
        let std_y: f32 = (ys.iter().map(|y| (y - mu_y).powi(2)).sum::<f32>() / n).sqrt();
        let r = if std_x > 1e-9 && std_y > 1e-9 {
            cov / (std_x * std_y)
        } else {
            0.0
        };
        Self {
            pearson_r: r.clamp(-1.0, 1.0),
        }
    }
}

/// Per-demographic fairness slice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessSlice {
    pub label: String,
    pub fmr: f32,
    pub fnmr: f32,
    pub sample_count: usize,
}

impl FairnessSlice {
    pub fn compute_by_skin_tone(corpus: &FrvtCorpus, threshold: f32) -> Vec<FairnessSlice> {
        use crate::corpus::SkinTone;
        let tones = [
            SkinTone::I,
            SkinTone::II,
            SkinTone::III,
            SkinTone::IV,
            SkinTone::V,
            SkinTone::VI,
        ];
        tones
            .iter()
            .filter_map(|tone| {
                let sub = FrvtCorpus {
                    images: corpus
                        .images
                        .iter()
                        .filter(|img| &img.skin_tone == tone)
                        .cloned()
                        .collect(),
                };
                if sub.images.is_empty() {
                    return None;
                }
                let m = FmrFnmr::compute(&sub, threshold);
                Some(FairnessSlice {
                    label: format!("{:?}", tone),
                    fmr: m.fmr,
                    fnmr: m.fnmr,
                    sample_count: sub.images.len(),
                })
            })
            .collect()
    }
}
