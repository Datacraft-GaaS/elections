//! GPU matcher trait and stub implementation.

use crate::domain::{AcceleratorCapabilities, AcceleratorKind, MatchQuery, Score};
use crate::error::AcceleratorError;

pub trait GpuMatcher: Send + Sync {
    fn capabilities(&self) -> AcceleratorCapabilities;
    fn batch_1n(&self, query: &MatchQuery) -> Result<Vec<Score>, AcceleratorError>;
    fn health(&self) -> bool;
}

pub struct StubGpuMatcher {
    pub kind: AcceleratorKind,
}

impl StubGpuMatcher {
    pub fn new(kind: AcceleratorKind) -> Self {
        Self { kind }
    }
}

impl GpuMatcher for StubGpuMatcher {
    fn capabilities(&self) -> AcceleratorCapabilities {
        AcceleratorCapabilities {
            kind: self.kind.clone(),
            max_batch_size: 10_000,
            throughput_kps: 500.0,
            power_w: 250.0,
            healthy: true,
        }
    }

    fn batch_1n(&self, query: &MatchQuery) -> Result<Vec<Score>, AcceleratorError> {
        if !self.health() {
            return Err(AcceleratorError::Unhealthy);
        }
        if query.gallery_ids.len() != query.gallery_templates.len() {
            return Err(AcceleratorError::TemplateSizeMismatch);
        }
        Ok(query
            .gallery_ids
            .iter()
            .map(|&gid| Score {
                probe_id: query.probe_id,
                gallery_id: gid,
                score: 0.5,
            })
            .collect())
    }

    fn health(&self) -> bool {
        true
    }
}
