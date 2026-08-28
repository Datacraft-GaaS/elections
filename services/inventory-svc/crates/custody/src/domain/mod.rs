use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Asset types tracked in custody.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    #[default]
    /// variant.
    Kit,
    /// variant.
    Ballot,
    /// variant.
    Seal,
    /// variant.
    Consumable,
}

/// Lifecycle status of a managed asset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    #[default]
    /// variant.
    New,
    /// variant.
    InTransit,
    /// variant.
    InWarehouse,
    /// variant.
    Provisioned,
    /// variant.
    Deployed,
    /// variant.
    Returned,
    /// variant.
    Faulty,
    /// variant.
    InRepair,
    /// variant.
    Decommissioned,
    /// variant.
    Destroyed,
    /// variant.
    Donated,
    /// variant.
    EvidentiaryHold,
}

/// A tracked physical asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// asset_id.
    pub asset_id: String,
    /// asset_type.
    pub asset_type: AssetType,
    /// serial.
    pub serial: String,
    /// model.
    pub model: String,
    /// status.
    pub status: AssetStatus,
    /// station_code.
    pub station_code: Option<String>,
    /// last_seen_at.
    pub last_seen_at: DateTime<Utc>,
}

/// A custody-chain event recording transfer of responsibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyEvent {
    /// event_id.
    pub event_id: String,
    /// asset_id.
    pub asset_id: String,
    /// from_custodian.
    pub from_custodian: String,
    /// to_custodian.
    pub to_custodian: String,
    /// location.
    pub location: String,
    /// HLC timestamp in milliseconds since Unix epoch.
    pub ts_hlc: i64,
    /// signature.
    pub signature: String,
    /// seq.
    pub seq: u64,
}

/// A tamper-evident seal record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seal {
    /// seal_id.
    pub seal_id: String,
    /// asset_id.
    pub asset_id: String,
    /// seal_type.
    pub seal_type: String,
    /// affixed_at.
    pub affixed_at: DateTime<Utc>,
    /// affixed_by.
    pub affixed_by: String,
    /// broken_at.
    pub broken_at: Option<DateTime<Utc>>,
    /// broken_reason.
    pub broken_reason: Option<String>,
    /// broken_by.
    pub broken_by: Option<String>,
    /// signature.
    pub signature: String,
}

/// Consumable supply item tracked per station.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumable {
    /// item_id.
    pub item_id: String,
    /// consumable_type.
    pub consumable_type: String,
    /// quantity.
    pub quantity: i64,
    /// unit.
    pub unit: String,
    /// station_code.
    pub station_code: String,
    /// replenishment_threshold.
    pub replenishment_threshold: i64,
}

/// Reserve pool for a category of assets within a scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservePool {
    /// pool_id.
    pub pool_id: String,
    /// scope.
    pub scope: String,
    /// asset_type.
    pub asset_type: String,
    /// count.
    pub count: i64,
    /// alert_threshold.
    pub alert_threshold: i64,
    /// updated_at.
    pub updated_at: DateTime<Utc>,
}

impl ReservePool {
    /// Returns true when the pool count has fallen to or below the alert threshold.
    pub fn is_below_threshold(&self) -> bool {
        self.count <= self.alert_threshold
    }
}
