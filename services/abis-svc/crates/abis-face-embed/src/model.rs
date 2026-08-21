//! EmbeddingModel enum: loss family + ONNX model path.

use serde::{Deserialize, Serialize};

/// Recognised loss families for face recognition training.
///
/// The interface is identical across families; only the training objective
/// (and therefore generalisation characteristics) differ.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LossFamily {
    /// ArcFace additive angular margin (Deng et al., CVPR 2019).
    ArcFace,
    /// AdaFace adaptive margin (Kim et al., CVPR 2022).
    AdaFace,
    /// CosFace large margin cosine (Wang et al., CVPR 2018).
    CosFace,
}

/// Embedding model descriptor: loss family + path to the ONNX model file.
///
/// When `onnx_path` is `None`, the hash-based baseline embedder is used
/// (identical inputs produce identical outputs; useful for CI).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModel {
    pub family: LossFamily,
    pub onnx_path: Option<std::path::PathBuf>,
}

impl EmbeddingModel {
    pub fn arcface(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            family: LossFamily::ArcFace,
            onnx_path: Some(path.into()),
        }
    }

    pub fn adaface(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            family: LossFamily::AdaFace,
            onnx_path: Some(path.into()),
        }
    }

    pub fn cosface(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            family: LossFamily::CosFace,
            onnx_path: Some(path.into()),
        }
    }

    /// Fallback hash-based baseline (no ONNX model file required).
    pub fn baseline(family: LossFamily) -> Self {
        Self {
            family,
            onnx_path: None,
        }
    }
}
