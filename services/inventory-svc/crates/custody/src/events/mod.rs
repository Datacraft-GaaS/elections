use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// KIT_DISPATCHED.
pub const KIT_DISPATCHED: &str = "eel.custody.kit_dispatched.v1";
/// KIT_RECEIVED.
pub const KIT_RECEIVED: &str = "eel.custody.kit_received.v1";
/// SEAL_BROKEN.
pub const SEAL_BROKEN: &str = "eel.custody.seal_broken.v1";
/// CONSUMABLE_USED.
pub const CONSUMABLE_USED: &str = "eel.custody.consumable_used.v1";
/// RESERVE_POOL_UPDATED.
pub const RESERVE_POOL_UPDATED: &str = "eel.custody.reserve_pool_updated.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
/// KitDispatched.
pub struct KitDispatched {
    /// asset_id.
    pub asset_id: String,
    /// from_custodian.
    pub from_custodian: String,
    /// to_custodian.
    pub to_custodian: String,
    /// location.
    pub location: String,
    /// dispatched_at.
    pub dispatched_at: DateTime<Utc>,
    /// seq.
    pub seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// KitReceived.
pub struct KitReceived {
    /// asset_id.
    pub asset_id: String,
    /// received_by.
    pub received_by: String,
    /// location.
    pub location: String,
    /// received_at.
    pub received_at: DateTime<Utc>,
    /// seq.
    pub seq: u64,
}

/// Breaking a seal requires two independent officers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealBroken {
    /// seal_id.
    pub seal_id: String,
    /// asset_id.
    pub asset_id: String,
    /// broken_by_officer_1.
    pub broken_by_officer_1: String,
    /// broken_by_officer_2.
    pub broken_by_officer_2: String,
    /// reason.
    pub reason: String,
    /// broken_at.
    pub broken_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// ConsumableUsed.
pub struct ConsumableUsed {
    /// item_id.
    pub item_id: String,
    /// quantity_used.
    pub quantity_used: i64,
    /// station_code.
    pub station_code: String,
    /// used_at.
    pub used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// ReservePoolUpdated.
pub struct ReservePoolUpdated {
    /// pool_id.
    pub pool_id: String,
    /// old_count.
    pub old_count: i64,
    /// new_count.
    pub new_count: i64,
    /// updated_at.
    pub updated_at: DateTime<Utc>,
}
