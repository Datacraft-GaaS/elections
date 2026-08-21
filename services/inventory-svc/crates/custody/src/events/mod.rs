use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const KIT_DISPATCHED: &str = "eel.custody.kit_dispatched.v1";
pub const KIT_RECEIVED: &str = "eel.custody.kit_received.v1";
pub const SEAL_BROKEN: &str = "eel.custody.seal_broken.v1";
pub const CONSUMABLE_USED: &str = "eel.custody.consumable_used.v1";
pub const RESERVE_POOL_UPDATED: &str = "eel.custody.reserve_pool_updated.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitDispatched {
    pub asset_id: String,
    pub from_custodian: String,
    pub to_custodian: String,
    pub location: String,
    pub dispatched_at: DateTime<Utc>,
    pub seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitReceived {
    pub asset_id: String,
    pub received_by: String,
    pub location: String,
    pub received_at: DateTime<Utc>,
    pub seq: u64,
}

/// Breaking a seal requires two independent officers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealBroken {
    pub seal_id: String,
    pub asset_id: String,
    pub broken_by_officer_1: String,
    pub broken_by_officer_2: String,
    pub reason: String,
    pub broken_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableUsed {
    pub item_id: String,
    pub quantity_used: i64,
    pub station_code: String,
    pub used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservePoolUpdated {
    pub pool_id: String,
    pub old_count: i64,
    pub new_count: i64,
    pub updated_at: DateTime<Utc>,
}
