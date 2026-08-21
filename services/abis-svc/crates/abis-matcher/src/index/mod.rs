//! Template index abstraction and implementations.

pub mod bloom_prune;
pub mod hnsw;
pub mod shard;

pub use bloom_prune::DemographicBloom;
pub use hnsw::HnswIndex;
pub use shard::{shard_id, ShardId};

use thiserror::Error;

/// A candidate returned by index search.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub subject_id: String,
    /// Distance to probe (lower = closer).
    pub distance: f32,
}

/// Errors produced by index operations.
#[derive(Debug, Error)]
pub enum IndexError {
    #[error("subject not found: {0}")]
    NotFound(String),
    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimMismatch { expected: usize, got: usize },
}

/// Generic template index contract.
pub trait TemplateIndex: Send + Sync {
    /// Insert a subject's embedding.
    fn add(&mut self, subject_id: &str, embedding: &[f32]) -> Result<(), IndexError>;

    /// Remove a subject's embedding.
    fn remove(&mut self, subject_id: &str) -> Result<(), IndexError>;

    /// Approximate nearest-neighbour search. Returns up to `k` candidates.
    fn search(&self, probe: &[f32], k: usize) -> Result<Vec<Candidate>, IndexError>;

    /// Number of entries in the index.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
