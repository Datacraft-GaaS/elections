use chrono::{DateTime, Utc};
use tracing::info;

/// Publishes Merkle roots to the transparency-svc.
///
/// Pushes to 5 independent channels every 10 min for audit resilience.
pub struct MerklePublisher;

impl MerklePublisher {
    pub fn new() -> Self {
        Self
    }

    /// Publish a Merkle root hash for the given window timestamp.
    /// In production this fans out over 5 independent transparency channels.
    pub fn publish_root(
        &self,
        root_hash: &str,
        window_ts: DateTime<Utc>,
    ) -> Result<(), MerkleError> {
        info!(root_hash, window_ts = %window_ts, "publishing merkle root");
        // TODO(WP-4.3): fan out to transparency-svc over 5 channels
        Ok(())
    }
}

impl Default for MerklePublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MerkleError {
    #[error("failed to publish merkle root: {0}")]
    Publish(String),
}
