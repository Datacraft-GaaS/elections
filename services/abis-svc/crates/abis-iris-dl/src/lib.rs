//! Deep iris embedding via hash-based deterministic embedder (DeepIrisNet-inspired).
//!
//! Reference: Gangwar, A. & Joshi, A. (2016). DeepIrisNet: Deep iris representation
//! with applications in iris recognition and cross-sensor iris recognition.
//! IEEE ICIP 2016.
#![warn(missing_docs)]

pub mod embedder;
pub mod hash_embedder;

pub use embedder::{EmbedError, Embedding, IrisEmbedder};
