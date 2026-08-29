//! Fingerprint ridge skeleton graph matching.
//!
//! Pipeline:
//! 1. Binarise grayscale input.
//! 2. Zhang-Suen thinning -> 1-pixel-wide skeleton.
//! 3. Build a graph: nodes = ridge endpoints + bifurcations; edges = ridge segments with
//!    geometric attributes (length, curvature, start/end orientation).
//! 4. Match via approximate subgraph isomorphism (VF2 for small graphs, greedy for large).
#![warn(missing_docs)]

pub mod graph;
pub mod matcher;
pub mod skeleton;

pub use graph::{skeleton_to_graph, RidgeEdge, RidgeGraph, RidgeNode, NodeKind};
pub use matcher::SkeletonMatcher;
pub use skeleton::{extract_skeleton, BinaryImage, RidgeSkeleton};

use thiserror::Error;

/// Errors produced by this crate.
#[derive(Debug, Error)]
pub enum SkeletonError {
    #[error("image too small (minimum 8x8)")]
    ImageTooSmall,
    #[error("invalid image dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    #[error("graph matching failed: {0}")]
    MatchingFailed(String),
}
