use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum TlogError {
    #[error("tlog append failed: {0}")] Append(String),
    #[error("tlog rpc error: {0}")] Rpc(String),
}

pub struct TlogClient { endpoint: String }
impl TlogClient {
    pub fn new(endpoint: impl Into<String>) -> Self { Self { endpoint: endpoint.into() } }
    pub async fn append_leaf(&self, merkle_root: &str, snapshot_id: &str) -> Result<(), TlogError> {
        // TODO: wire tonic gRPC to transparency-svc AppendLeaf when proto is available
        info!(endpoint = %self.endpoint, merkle_root, snapshot_id, "tlog_client: stub AppendLeaf");
        Ok(())
    }
}
