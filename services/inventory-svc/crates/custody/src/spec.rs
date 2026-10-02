// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Public contract models (custody/openapi.yaml).

use serde::{Deserialize, Serialize};

/// Public custody asset view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// asset_id.
    pub asset_id: String,
    /// asset_type.
    pub asset_type: String,
    /// serial_number.
    pub serial_number: String,
    /// current_custodian_id.
    pub current_custodian_id: String,
    /// current_location.
    pub current_location: String,
    /// status.
    pub status: String,
    /// condition.
    pub condition: String,
    /// created_at.
    pub created_at: String,
    /// updated_at.
    pub updated_at: String,
}

/// Request body for registering an asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCreate {
    /// asset_type.
    pub asset_type: String,
    /// serial_number.
    pub serial_number: String,
    /// current_custodian_id.
    pub current_custodian_id: String,
    /// current_location.
    pub current_location: String,
    /// condition.
    pub condition: String,
}

/// Public custody transfer view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyTransfer {
    /// transfer_id.
    pub transfer_id: String,
    /// asset_id.
    pub asset_id: String,
    /// from_custodian_id.
    pub from_custodian_id: String,
    /// to_custodian_id.
    pub to_custodian_id: String,
    /// from_location.
    pub from_location: String,
    /// to_location.
    pub to_location: String,
    /// reason.
    pub reason: String,
    /// transferred_at.
    pub transferred_at: String,
    /// officer_id.
    pub officer_id: String,
    /// witness_id.
    pub witness_id: Option<String>,
}

/// Request body for transferring custody.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyTransferCreate {
    /// to_custodian_id.
    pub to_custodian_id: String,
    /// to_location.
    pub to_location: String,
    /// reason.
    pub reason: String,
    /// officer_id.
    pub officer_id: String,
    /// witness_id.
    pub witness_id: Option<String>,
}

/// Public seal view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seal {
    /// seal_id.
    pub seal_id: String,
    /// asset_id.
    pub asset_id: String,
    /// seal_number.
    pub seal_number: String,
    /// sealed_by_id.
    pub sealed_by_id: String,
    /// sealed_at.
    pub sealed_at: String,
    /// status.
    pub status: String,
}

/// Request body for creating a seal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealCreate {
    /// asset_id.
    pub asset_id: String,
    /// seal_number.
    pub seal_number: String,
    /// sealed_by_id.
    pub sealed_by_id: String,
}

/// Public seal-break view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealBreak {
    /// break_id.
    pub break_id: String,
    /// seal_id.
    pub seal_id: String,
    /// broken_by_id.
    pub broken_by_id: String,
    /// countersign_id.
    pub countersign_id: String,
    /// reason.
    pub reason: String,
    /// broken_at.
    pub broken_at: String,
}

/// Request body for breaking a seal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealBreakCreate {
    /// broken_by_id.
    pub broken_by_id: String,
    /// countersign_id.
    pub countersign_id: String,
    /// reason.
    pub reason: String,
}

/// Public reserve view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reserve {
    /// reserve_id.
    pub reserve_id: String,
    /// asset_type.
    pub asset_type: String,
    /// location.
    pub location: String,
    /// quantity.
    pub quantity: u32,
    /// min_threshold.
    pub min_threshold: u32,
    /// status.
    pub status: String,
}

/// Public custody summary view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodySummary {
    /// total_assets.
    pub total_assets: u32,
    /// assets_in_custody.
    pub assets_in_custody: u32,
    /// assets_in_transit.
    pub assets_in_transit: u32,
    /// broken_seals_today.
    pub broken_seals_today: u32,
    /// discrepancy_count.
    pub discrepancy_count: u32,
    /// generated_at.
    pub generated_at: String,
}

/// One reconciliation discrepancy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrepancy {
    /// asset_type.
    pub asset_type: String,
    /// serial_number.
    pub serial_number: Option<String>,
    /// issue.
    pub issue: String,
    /// notes.
    pub notes: Option<String>,
}

/// Public reconciliation report view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcileReport {
    /// reconcile_id.
    pub reconcile_id: String,
    /// station_code.
    pub station_code: String,
    /// election_id.
    pub election_id: String,
    /// assets_expected.
    pub assets_expected: u32,
    /// assets_found.
    pub assets_found: u32,
    /// discrepancies.
    pub discrepancies: Vec<Discrepancy>,
    /// status.
    pub status: String,
    /// officer_id.
    pub officer_id: String,
}

/// Request body for starting reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcileCreate {
    /// station_code.
    pub station_code: String,
    /// election_id.
    pub election_id: String,
    /// assets_expected.
    pub assets_expected: u32,
    /// assets_found.
    pub assets_found: u32,
    /// discrepancies.
    pub discrepancies: Vec<Discrepancy>,
    /// officer_id.
    pub officer_id: String,
}
