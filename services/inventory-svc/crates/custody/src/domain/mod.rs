use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Asset types tracked in custody.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Kit,
    Ballot,
    Seal,
    Consumable,
}

/// Lifecycle status of a managed asset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    New,
    InTransit,
    InWarehouse,
    Provisioned,
    Deployed,
    Returned,
    Faulty,
    InRepair,
    Decommissioned,
    Destroyed,
    Donated,
    EvidentiaryHold,
}

/// A tracked physical asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub asset_id: String,
    pub asset_type: AssetType,
    pub serial: String,
    pub model: String,
    pub status: AssetStatus,
    pub station_code: Option<String>,
    pub last_seen_at: DateTime<Utc>,
}

/// A custody-chain event recording transfer of responsibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyEvent {
    pub event_id: String,
    pub asset_id: String,
    pub from_custodian: String,
    pub to_custodian: String,
    pub location: String,
    /// HLC timestamp in milliseconds since Unix epoch.
    pub ts_hlc: i64,
    pub signature: String,
    pub seq: u64,
}

/// A tamper-evident seal record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seal {
    pub seal_id: String,
    pub asset_id: String,
    pub seal_type: String,
    pub affixed_at: DateTime<Utc>,
    pub affixed_by: String,
    pub broken_at: Option<DateTime<Utc>>,
    pub broken_reason: Option<String>,
    pub broken_by: Option<String>,
    pub signature: String,
}

/// Consumable supply item tracked per station.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumable {
    pub item_id: String,
    pub consumable_type: String,
    pub quantity: i64,
    pub unit: String,
    pub station_code: String,
    pub replenishment_threshold: i64,
}

/// Reserve pool for a category of assets within a scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservePool {
    pub pool_id: String,
    pub scope: String,
    pub asset_type: String,
    pub count: i64,
    pub alert_threshold: i64,
    pub updated_at: DateTime<Utc>,
}

impl ReservePool {
    /// Returns true when the pool count has fallen to or below the alert threshold.
    pub fn is_below_threshold(&self) -> bool {
        self.count <= self.alert_threshold
    }
}
