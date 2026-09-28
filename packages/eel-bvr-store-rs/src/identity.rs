//! Instance identity: the unique id stamped on everything a kit captures.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A registration station instance.
///
/// The id is a UUIDv7, so it is both unique and time-ordered, which keeps
/// insertion order stable when a fleet's records are merged. It is generated
/// once at first run and persisted; it is **not** derived from hardware, so a
/// re-imaged or cloned disk does not silently inherit a previous identity. A
/// fleet administrator reconciling a cloned image sees two ids and can tell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceIdentity {
    /// UUIDv7, unique per client instance.
    pub instance_id: Uuid,
    /// Human-readable station label, e.g. "Nairobi County 04 / Kit 12".
    pub station_label: String,
    /// 32-byte Ed25519 public key, base64-encoded. Safe to publish.
    pub public_key_b64: String,
}

impl InstanceIdentity {
    /// Returns the instance id as a canonical hyphenated string.
    pub fn id_string(&self) -> String {
        self.instance_id.to_string()
    }
}
