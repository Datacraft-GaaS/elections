//! Random Forest model for NFIQ 2.2 quality scoring.
//!
//! The model is JSON-serialised trees (see `DEFAULT_MODEL` below for a minimal
//! synthetic default used in tests).  At runtime, the NIST-published model can
//! be loaded from a file via `RandomForestModel::from_path`.
//!
//! Reference: NIST SP 500-333, Section 5.

use crate::Nfiq22Error;
use serde::{Deserialize, Serialize};

/// A single decision tree node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TreeNode {
    /// Internal (split) node.
    #[serde(rename = "split")]
    Split {
        /// Feature index (0..14).
        feature: usize,
        /// Split threshold.
        threshold: f32,
        /// Left child index (feature < threshold).
        left: usize,
        /// Right child index (feature >= threshold).
        right: usize,
    },
    /// Leaf node with predicted quality class.
    #[serde(rename = "leaf")]
    Leaf {
        /// Quality class [0, 100].
        value: f32,
    },
}

/// A single decision tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTree {
    pub nodes: Vec<TreeNode>,
}

impl DecisionTree {
    /// Predict quality for a feature vector.
    pub fn predict(&self, features: &[f32; 14]) -> f32 {
        let mut idx = 0usize;
        loop {
            match &self.nodes[idx] {
                TreeNode::Leaf { value } => return *value,
                TreeNode::Split {
                    feature,
                    threshold,
                    left,
                    right,
                } => {
                    idx = if features[*feature] < *threshold {
                        *left
                    } else {
                        *right
                    };
                }
            }
        }
    }
}

/// Random Forest ensemble.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomForestModel {
    pub trees: Vec<DecisionTree>,
}

impl RandomForestModel {
    /// Predict quality [0, 100] as mean of tree predictions.
    pub fn predict(&self, features: &[f32; 14]) -> f32 {
        if self.trees.is_empty() {
            return 50.0;
        }
        let sum: f32 = self.trees.iter().map(|t| t.predict(features)).sum();
        (sum / self.trees.len() as f32).clamp(0.0, 100.0)
    }

    /// Load model from a JSON file.
    pub fn from_path(path: &std::path::Path) -> Result<Self, Nfiq22Error> {
        let data =
            std::fs::read_to_string(path).map_err(|e| Nfiq22Error::ModelLoad(e.to_string()))?;
        serde_json::from_str(&data).map_err(|e| Nfiq22Error::ModelLoad(e.to_string()))
    }

    /// Default small model trained on a synthetic corpus (for tests).
    /// Features: [lcs, ocl, ofl, roi, fda, gabor, mu, mmb, rvu, sigma, fingerjet_ose, gshear, rvfl, pix]
    pub fn default_model() -> Self {
        // Three minimal trees that split on LCS (feature 0), sigma (feature 9), and OCL (feature 1).
        // These are synthetic: real calibration requires the NIST-published model.
        let trees = vec![
            // Tree 0: splits on LCS (feature 0).
            DecisionTree {
                nodes: vec![
                    TreeNode::Split {
                        feature: 0,
                        threshold: 0.3,
                        left: 1,
                        right: 2,
                    },
                    TreeNode::Leaf { value: 30.0 },
                    TreeNode::Split {
                        feature: 0,
                        threshold: 0.6,
                        left: 3,
                        right: 4,
                    },
                    TreeNode::Leaf { value: 60.0 },
                    TreeNode::Leaf { value: 85.0 },
                ],
            },
            // Tree 1: splits on sigma (feature 9).
            DecisionTree {
                nodes: vec![
                    TreeNode::Split {
                        feature: 9,
                        threshold: 0.2,
                        left: 1,
                        right: 2,
                    },
                    TreeNode::Leaf { value: 20.0 },
                    TreeNode::Split {
                        feature: 9,
                        threshold: 0.5,
                        left: 3,
                        right: 4,
                    },
                    TreeNode::Leaf { value: 55.0 },
                    TreeNode::Leaf { value: 80.0 },
                ],
            },
            // Tree 2: splits on OCL (feature 1).
            DecisionTree {
                nodes: vec![
                    TreeNode::Split {
                        feature: 1,
                        threshold: 0.4,
                        left: 1,
                        right: 2,
                    },
                    TreeNode::Leaf { value: 25.0 },
                    TreeNode::Split {
                        feature: 1,
                        threshold: 0.7,
                        left: 3,
                        right: 4,
                    },
                    TreeNode::Leaf { value: 65.0 },
                    TreeNode::Leaf { value: 90.0 },
                ],
            },
        ];
        Self { trees }
    }
}
