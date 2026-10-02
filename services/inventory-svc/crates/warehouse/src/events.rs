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

//! Cross-service event contracts for the warehouse domain (M20 inventory,
//! design §8).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const WAREHOUSE_REGISTERED: &str = "eel.warehouse.warehouse_registered.v1";
pub const TRANSFER_CREATED: &str = "eel.warehouse.transfer_created.v1";
pub const TRANSFER_DEPARTED: &str = "eel.warehouse.transfer_departed.v1";
pub const TRANSFER_ARRIVED: &str = "eel.warehouse.transfer_arrived.v1";
pub const PALLET_RECEIVED: &str = "eel.warehouse.pallet_received.v1";
pub const PALLET_BREAK_BULK: &str = "eel.warehouse.pallet_break_bulk.v1";
pub const PALLET_DISPATCHED: &str = "eel.warehouse.pallet_dispatched.v1";
pub const PALLET_USED: &str = "eel.warehouse.pallet_used.v1";
pub const INCIDENT_OPENED: &str = "eel.warehouse.incident_opened.v1";
pub const INCIDENT_RESOLVED: &str = "eel.warehouse.incident_resolved.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseRegistered {
    pub warehouse_id: String,
    pub code: String,
    pub tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferDeparted {
    pub transfer_id: String,
    pub from_warehouse_id: String,
    pub to_warehouse_id: String,
    pub vehicle_id: Option<String>,
    pub departed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferArrived {
    pub transfer_id: String,
    pub to_warehouse_id: String,
    pub missing_pallets: Vec<String>,
    pub extra_pallets: Vec<String>,
    pub arrived_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalletReceived {
    pub pallet_id: String,
    pub serial: String,
    pub warehouse_id: String,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalletBreakBulk {
    pub pallet_id: String,
    pub item_count: u64,
    pub break_bulk_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalletUsed {
    pub pallet_id: String,
    pub station_code: String,
    pub used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentOpened {
    pub incident_id: String,
    pub incident_type: String,
    /// "P0" | "P1" | "P2" | "P3".
    pub severity: String,
    pub summary: String,
    pub reported_at: DateTime<Utc>,
}
