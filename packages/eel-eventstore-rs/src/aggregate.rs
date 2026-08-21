use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::error::EventStoreError;

/// A single event as stored in `eel_events`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredEvent {
    pub stream_id: Uuid,
    pub stream_version: i64,
    pub id: Uuid,
    pub event_type: String,
    pub payload: Value,
    pub prev_hash: Option<String>,
    pub hlc: String,
    pub recorded_at: DateTime<Utc>,
}

/// Snapshot of an aggregate's state at a given stream version.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AggregateSnapshot {
    pub stream_id: Uuid,
    pub stream_version: i64,
    pub state: Value,
    pub taken_at: DateTime<Utc>,
}

/// Trait implemented by each domain aggregate.
///
/// Implementors fold `StoredEvent`s into their state via [`apply`] and expose
/// a point-in-time snapshot via [`snapshot`].
pub trait Aggregate: Send + Sync {
    /// Fold a single event into the aggregate's state.
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError>;

    /// Emit a serialisable snapshot of the current state.
    fn snapshot(&self) -> AggregateSnapshot;

    /// The globally unique identifier for this aggregate's stream.
    fn stream_id(&self) -> Uuid;
}
