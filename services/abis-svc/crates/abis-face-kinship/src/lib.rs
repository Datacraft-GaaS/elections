//! Kinship verification and family cluster grouping.
//!
//! Reference:
//! - Lu, J. et al. (2014). "Kinship Verification with Deep Convolutional Neural
//!   Networks." BMVC 2014. (embedding-based kinship scoring)
//! - Guo, G. & Wang, X. (2012). "Kinship Verification Using Facial Images
//!   by Regularized Regression." CVPR 2012. (similarity-based approach)
//!
//! Kinship is estimated purely from embedding cosine similarity here, which is
//! a baseline; a full system would use a dedicated kinship metric learned from
//! family photo datasets.
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const EMBED_DIM: usize = 512;

/// Kinship relationship label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Relationship {
    Parent,
    Sibling,
    Twin,
    Other,
}

impl std::fmt::Display for Relationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parent => write!(f, "parent"),
            Self::Sibling => write!(f, "sibling"),
            Self::Twin => write!(f, "twin"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Kinship error.
#[derive(Debug, Error)]
pub enum KinshipError {
    #[error("embedding dimension mismatch: expected {EMBED_DIM}")]
    DimensionMismatch,
}

/// Kinship prediction for a pair of embeddings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KinshipPrediction {
    /// Probability of kinship in [0.0, 1.0].
    pub p_kin: f32,
    /// Most probable relationship label.
    pub relationship: Relationship,
}

/// Kinship classifier operating on face embeddings.
///
/// Uses cosine similarity as the primary signal. Thresholds are empirical
/// baselines; a production system would require labelled family datasets.
#[derive(Debug, Clone)]
pub struct KinshipClassifier {
    /// Cosine similarity >= this => classified as kin.
    pub kin_threshold: f32,
    /// Cosine similarity >= this => classified as twin.
    pub twin_threshold: f32,
}

impl Default for KinshipClassifier {
    fn default() -> Self {
        Self {
            kin_threshold: 0.5,
            twin_threshold: 0.95,
        }
    }
}

impl KinshipClassifier {
    pub fn new(kin_threshold: f32, twin_threshold: f32) -> Self {
        Self {
            kin_threshold,
            twin_threshold,
        }
    }

    fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| x * y)
            .sum::<f32>()
            .clamp(-1.0, 1.0)
    }

    /// Predict kinship between two embeddings.
    pub fn predict(&self, a: &[f32], b: &[f32]) -> Result<KinshipPrediction, KinshipError> {
        if a.len() != EMBED_DIM || b.len() != EMBED_DIM {
            return Err(KinshipError::DimensionMismatch);
        }
        let sim = Self::cosine_sim(a, b);
        let p_kin = ((sim - self.kin_threshold) / (1.0 - self.kin_threshold)).clamp(0.0, 1.0);

        let relationship = if sim >= self.twin_threshold {
            Relationship::Twin
        } else if sim >= self.kin_threshold + (self.twin_threshold - self.kin_threshold) * 0.7 {
            Relationship::Sibling
        } else if sim >= self.kin_threshold {
            Relationship::Parent
        } else {
            Relationship::Other
        };

        Ok(KinshipPrediction {
            p_kin,
            relationship,
        })
    }
}

/// Family cluster: a set of subject IDs grouped as a family unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyCluster {
    pub cluster_id: usize,
    pub members: Vec<String>,
}

/// Build family clusters from a list of (id, embedding) pairs using greedy
/// transitivity: if A and B are kin AND B and C are kin, group {A, B, C}.
///
/// Reference: Disjoint-set union (union-find) grouping.
pub fn cluster_families(
    subjects: &[(String, Vec<f32>)],
    classifier: &KinshipClassifier,
) -> Vec<FamilyCluster> {
    let n = subjects.len();
    if n == 0 {
        return Vec::new();
    }

    // Union-find parent array.
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut Vec<usize>, x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }

    for i in 0..n {
        for j in (i + 1)..n {
            let pred = classifier.predict(&subjects[i].1, &subjects[j].1);
            if let Ok(p) = pred {
                if p.p_kin >= 0.5 {
                    let ri = find(&mut parent, i);
                    let rj = find(&mut parent, j);
                    if ri != rj {
                        parent[rj] = ri;
                    }
                }
            }
        }
    }

    // Collect clusters.
    let mut groups: std::collections::HashMap<usize, Vec<String>> =
        std::collections::HashMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        groups.entry(root).or_default().push(subjects[i].0.clone());
    }

    groups
        .into_values()
        .enumerate()
        .map(|(id, members)| FamilyCluster {
            cluster_id: id,
            members,
        })
        .collect()
}
