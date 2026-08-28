//! Public contract models (custody/openapi.yaml).

use serde::{Deserialize, Serialize};

/// Public custody asset view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub asset_id: String,
    pub asset_type: String,
    pub serial_number: String,
    pub current_custodian_id: String,
    pub current_location: String,
    pub status: String,
    pub condition: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for registering an asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCreate {
    pub asset_type: String,
    pub serial_number: String,
    pub current_custodian_id: String,
    pub current_location: String,
    pub condition: String,
}

/// Public custody transfer view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyTransfer {
    pub transfer_id: String,
    pub asset_id: String,
    pub from_custodian_id: String,
    pub to_custodian_id: String,
    pub from_location: String,
    pub to_location: String,
    pub reason: String,
    pub transferred_at: String,
    pub officer_id: String,
    pub witness_id: Option<String>,
}

/// Request body for transferring custody.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyTransferCreate {
    pub to_custodian_id: String,
    pub to_location: String,
    pub reason: String,
    pub officer_id: String,
    pub witness_id: Option<String>,
}

/// Public seal view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seal {
    pub seal_id: String,
    pub asset_id: String,
    pub seal_number: String,
    pub sealed_by_id: String,
    pub sealed_at: String,
    pub status: String,
}

/// Request body for creating a seal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealCreate {
    pub asset_id: String,
    pub seal_number: String,
    pub sealed_by_id: String,
}

/// Public seal-break view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealBreak {
    pub break_id: String,
    pub seal_id: String,
    pub broken_by_id: String,
    pub countersign_id: String,
    pub reason: String,
    pub broken_at: String,
}

/// Request body for breaking a seal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealBreakCreate {
    pub broken_by_id: String,
    pub countersign_id: String,
    pub reason: String,
}

/// Public reserve view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reserve {
    pub reserve_id: String,
    pub asset_type: String,
    pub location: String,
    pub quantity: u32,
    pub min_threshold: u32,
    pub status: String,
}

/// Public custody summary view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodySummary {
    pub total_assets: u32,
    pub assets_in_custody: u32,
    pub assets_in_transit: u32,
    pub broken_seals_today: u32,
    pub discrepancy_count: u32,
    pub generated_at: String,
}

/// One reconciliation discrepancy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrepancy {
    pub asset_type: String,
    pub serial_number: Option<String>,
    pub issue: String,
    pub notes: Option<String>,
}

/// Public reconciliation report view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcileReport {
    pub reconcile_id: String,
    pub station_code: String,
    pub election_id: String,
    pub assets_expected: u32,
    pub assets_found: u32,
    pub discrepancies: Vec<Discrepancy>,
    pub status: String,
    pub officer_id: String,
}

/// Request body for starting reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcileCreate {
    pub station_code: String,
    pub election_id: String,
    pub assets_expected: u32,
    pub assets_found: u32,
    pub discrepancies: Vec<Discrepancy>,
    pub officer_id: String,
}
