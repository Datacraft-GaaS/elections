// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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

pub use graph::{skeleton_to_graph, NodeKind, RidgeEdge, RidgeGraph, RidgeNode};
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
