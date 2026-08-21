//! Deep-learning fingerprint matcher via ONNX Runtime.
//!
//! Implements a DeepPrint-style 384-dimensional embedding + cosine-similarity
//! matching pipeline.
//!
//! Reference: "Learning a Fixed-Length Fingerprint Representation," Engelsma et al.,
//! IEEE TPAMI 2021.  ONNX model acquired via the DeepPrint project or equivalent.
//!
//! When the `onnx` feature is disabled or the model file is absent, a deterministic
//! hash-based pseudo-embedding is used so the crate remains testable without a GPU
//! or model artefact.

pub mod embed;
pub mod fusion;
pub mod matcher;
pub mod pool;

pub use embed::{DeepPrintEmbedder, Embedding};
pub use matcher::DlMatcher;

/// Gray-scale fingerprint image (8-bit pixels, row-major).
#[derive(Debug, Clone)]
pub struct GrayImage {
	pub pixels: Vec<u8>,
	pub width: u32,
	pub height: u32,
}

impl GrayImage {
	pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> Self {
		debug_assert_eq!(pixels.len(), (width * height) as usize);
		Self { pixels, width, height }
	}
}

/// Error type for the DL matcher.
#[derive(Debug, thiserror::Error)]
pub enum DlError {
	#[error("ONNX session error: {0}")]
	SessionError(String),
	#[error("embedding dimension mismatch: expected 384, got {0}")]
	DimMismatch(usize),
	#[error("model path not set; set EEL_FP_DL_ONNX_PATH or enable feature `onnx`")]
	ModelNotConfigured,
	#[error("pool exhausted")]
	PoolExhausted,
}

/// Embedding dimension for the DeepPrint model.
pub const EMBED_DIM: usize = 384;
