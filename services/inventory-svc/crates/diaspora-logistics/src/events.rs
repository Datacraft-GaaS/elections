use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const CONTAINER_DISPATCHED: &str = "eel.diaspora.container_dispatched";
pub const CONTAINER_RECEIVED: &str = "eel.diaspora.container_received";
pub const SEAL_BROKEN: &str = "eel.diaspora.seal_broken";
pub const RECONCILIATION_COMPLETE: &str = "eel.diaspora.reconciliation_complete";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDispatchedPayload {
    pub container_id: String,
    pub actor_id: String,
    pub location: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealBrokenPayload {
    pub container_id: String,
    pub seal_id: String,
    pub broken_by: String,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}
