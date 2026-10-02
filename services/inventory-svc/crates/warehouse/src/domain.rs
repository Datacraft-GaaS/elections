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

//! Warehouse, transfer, pallet, and incident domain (M20 inventory).
//!
//! Four related concerns for moving and accounting for election material:
//!
//! 1. **Warehouses** — a tiered network (master / regional / county) with a
//!    geofence, so custody locations can be validated.
//! 2. **Transfers** — inter-warehouse movements with explicit departure and
//!    arrival, plus arrival reconciliation (missing/extra pallets).
//! 3. **Pallets** — a pallet carries a serial number and is tracked from
//!    receipt, through storage and transfer, to break bulk and use at the
//!    polling station.
//! 4. **Incidents** — tamper, loss, damage, and discrepancy notes with a
//!    severity and an append-only note log.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarehouseTier {
    #[default]
    Master,
    Regional,
    County,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferStatus {
    #[default]
    Created,
    Departed,
    Arrived,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PalletStatus {
    #[default]
    Received,
    InStorage,
    InTransit,
    BreakBulk,
    Dispatched,
    Used,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    #[default]
    BallotBox,
    BallotLot,
    Kit,
    Consumable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentType {
    #[default]
    Tamper,
    Loss,
    Damage,
    Discrepancy,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IncidentSeverity {
    #[default]
    #[serde(rename = "P0")]
    P0,
    #[serde(rename = "P1")]
    P1,
    #[serde(rename = "P2")]
    P2,
    #[serde(rename = "P3")]
    P3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentStatus {
    #[default]
    Open,
    Investigating,
    Resolved,
}

// ---------------------------------------------------------------------------
// Warehouses
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warehouse {
    pub id: String,
    pub code: String,
    pub name: String,
    pub tier: WarehouseTier,
    pub latitude: f64,
    pub longitude: f64,
    pub geofence_radius_m: f64,
}

impl Warehouse {
    pub fn new(
        id: String,
        code: String,
        name: String,
        tier: WarehouseTier,
        latitude: f64,
        longitude: f64,
        geofence_radius_m: f64,
    ) -> Result<Self, WarehouseError> {
        if geofence_radius_m <= 0.0 {
            return Err(WarehouseError::InvalidGeofence {
                radius: geofence_radius_m,
            });
        }
        Ok(Self {
            id,
            code,
            name,
            tier,
            latitude,
            longitude,
            geofence_radius_m,
        })
    }

    /// True when `(lat, lon)` falls within this warehouse's geofence.
    pub fn contains(&self, latitude: f64, longitude: f64) -> bool {
        // Local planar approximation is fine for a geofence of a few hundred
        // metres; a production check uses the boundary-svc polygon.
        let dx = (longitude - self.longitude) * 111_320.0 * self.latitude.to_radians().cos();
        let dy = (latitude - self.latitude) * 110_540.0;
        (dx * dx + dy * dy) <= self.geofence_radius_m * self.geofence_radius_m
    }
}

// ---------------------------------------------------------------------------
// Transfers
// ---------------------------------------------------------------------------

/// A movement of pallets between two warehouses, with explicit departure and
/// arrival tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseTransfer {
    pub id: String,
    pub from_warehouse_id: String,
    pub to_warehouse_id: String,
    pub vehicle_id: Option<String>,
    pub status: TransferStatus,
    pub expected_pallets: Vec<String>,
    pub arrived_pallets: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub departed_at: Option<DateTime<Utc>>,
    pub arrived_at: Option<DateTime<Utc>>,
}

impl WarehouseTransfer {
    pub fn new(
        id: String,
        from_warehouse_id: String,
        to_warehouse_id: String,
        vehicle_id: Option<String>,
        expected_pallets: Vec<String>,
        now: DateTime<Utc>,
    ) -> Result<Self, WarehouseError> {
        if from_warehouse_id == to_warehouse_id {
            return Err(WarehouseError::NoSelfTransfer);
        }
        Ok(Self {
            id,
            from_warehouse_id,
            to_warehouse_id,
            vehicle_id,
            status: TransferStatus::Created,
            expected_pallets,
            arrived_pallets: Vec::new(),
            created_at: now,
            departed_at: None,
            arrived_at: None,
        })
    }

    pub fn depart(&mut self, now: DateTime<Utc>) -> Result<(), WarehouseError> {
        self.expect(TransferStatus::Created, "depart")?;
        self.status = TransferStatus::Departed;
        self.departed_at = Some(now);
        Ok(())
    }

    /// Record arrival with the pallet serials actually received.
    pub fn arrive(
        &mut self,
        arrived: Vec<String>,
        now: DateTime<Utc>,
    ) -> Result<(), WarehouseError> {
        self.expect(TransferStatus::Departed, "arrive")?;
        self.status = TransferStatus::Arrived;
        self.arrived_pallets = arrived;
        self.arrived_at = Some(now);
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), WarehouseError> {
        self.expect(TransferStatus::Created, "cancel")?;
        self.status = TransferStatus::Cancelled;
        Ok(())
    }

    /// Expected pallets that did not arrive (reconciliation shortfall).
    pub fn missing_pallets(&self) -> Vec<String> {
        self.expected_pallets
            .iter()
            .filter(|p| !self.arrived_pallets.contains(p))
            .cloned()
            .collect()
    }

    /// Arrived pallets that were not on the manifest (reconciliation surplus).
    pub fn extra_pallets(&self) -> Vec<String> {
        self.arrived_pallets
            .iter()
            .filter(|p| !self.expected_pallets.contains(p))
            .cloned()
            .collect()
    }

    fn expect(&self, status: TransferStatus, action: &str) -> Result<(), WarehouseError> {
        if self.status != status {
            return Err(WarehouseError::InvalidTransition {
                from: transfer_status_str(self.status),
                action: action.to_string(),
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Pallets and break bulk
// ---------------------------------------------------------------------------

/// A pallet carrying ballot material, tracked from receipt to use at the
/// polling station. It may be broken into items (break bulk) mid-journey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pallet {
    pub id: String,
    /// Scannable serial number (barcode/RFID label).
    pub serial: String,
    pub warehouse_id: String,
    pub status: PalletStatus,
    pub items: Vec<PalletItem>,
    pub received_at: DateTime<Utc>,
    pub used_at_station: Option<String>,
}

/// One unit inside a pallet, split out during break bulk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalletItem {
    pub id: String,
    pub item_type: ItemType,
    pub reference_id: String,
    pub quantity: u64,
}

impl Pallet {
    pub fn receive(id: String, serial: String, warehouse_id: String, now: DateTime<Utc>) -> Self {
        Self {
            id,
            serial,
            warehouse_id,
            status: PalletStatus::Received,
            items: Vec::new(),
            received_at: now,
            used_at_station: None,
        }
    }

    pub fn store(&mut self) -> Result<(), WarehouseError> {
        self.expect(PalletStatus::Received, "store")?;
        self.status = PalletStatus::InStorage;
        Ok(())
    }

    pub fn start_transfer(&mut self) -> Result<(), WarehouseError> {
        self.expect(PalletStatus::InStorage, "start_transfer")?;
        self.status = PalletStatus::InTransit;
        Ok(())
    }

    /// Pallet arrives at a (possibly different) warehouse after transfer.
    pub fn complete_transfer(&mut self, warehouse_id: String) -> Result<(), WarehouseError> {
        self.expect(PalletStatus::InTransit, "complete_transfer")?;
        self.warehouse_id = warehouse_id;
        self.status = PalletStatus::InStorage;
        Ok(())
    }

    /// Break bulk: split the pallet into its constituent items.
    pub fn break_bulk(&mut self, items: Vec<PalletItem>) -> Result<(), WarehouseError> {
        self.expect(PalletStatus::InStorage, "break_bulk")?;
        if items.is_empty() {
            return Err(WarehouseError::EmptyBreakBulk);
        }
        self.items = items;
        self.status = PalletStatus::BreakBulk;
        Ok(())
    }

    pub fn dispatch(&mut self) -> Result<(), WarehouseError> {
        self.expect(PalletStatus::BreakBulk, "dispatch")?;
        self.status = PalletStatus::Dispatched;
        Ok(())
    }

    /// The pallet's contents were used at a polling station.
    pub fn mark_used(&mut self, station_code: String) -> Result<(), WarehouseError> {
        self.expect(PalletStatus::Dispatched, "mark_used")?;
        self.used_at_station = Some(station_code);
        self.status = PalletStatus::Used;
        Ok(())
    }

    fn expect(&self, status: PalletStatus, action: &str) -> Result<(), WarehouseError> {
        if self.status != status {
            return Err(WarehouseError::InvalidTransition {
                from: pallet_status_str(self.status),
                action: action.to_string(),
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Incidents
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub warehouse_id: Option<String>,
    pub pallet_id: Option<String>,
    pub transfer_id: Option<String>,
    pub incident_type: IncidentType,
    pub severity: IncidentSeverity,
    pub summary: String,
    pub status: IncidentStatus,
    pub reported_at: DateTime<Utc>,
    pub reported_by: String,
    pub resolved_at: Option<DateTime<Utc>>,
    pub notes: Vec<IncidentNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentNote {
    pub id: String,
    pub author: String,
    pub note: String,
    pub at: DateTime<Utc>,
}

impl Incident {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        warehouse_id: Option<String>,
        pallet_id: Option<String>,
        transfer_id: Option<String>,
        incident_type: IncidentType,
        severity: IncidentSeverity,
        summary: String,
        reported_by: String,
        now: DateTime<Utc>,
    ) -> Result<Self, WarehouseError> {
        if summary.trim().is_empty() {
            return Err(WarehouseError::EmptySummary);
        }
        Ok(Self {
            id,
            warehouse_id,
            pallet_id,
            transfer_id,
            incident_type,
            severity,
            summary,
            status: IncidentStatus::Open,
            reported_at: now,
            reported_by,
            resolved_at: None,
            notes: Vec::new(),
        })
    }

    pub fn add_note(&mut self, id: String, author: String, note: String, now: DateTime<Utc>) {
        self.notes.push(IncidentNote {
            id,
            author,
            note,
            at: now,
        });
    }

    pub fn resolve(&mut self, now: DateTime<Utc>) -> Result<(), WarehouseError> {
        if self.status == IncidentStatus::Resolved {
            return Err(WarehouseError::AlreadyResolved);
        }
        self.status = IncidentStatus::Resolved;
        self.resolved_at = Some(now);
        Ok(())
    }

    pub fn investigate(&mut self) -> Result<(), WarehouseError> {
        if self.status != IncidentStatus::Open {
            return Err(WarehouseError::InvalidTransition {
                from: incident_status_str(self.status),
                action: "investigate".to_string(),
            });
        }
        self.status = IncidentStatus::Investigating;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error, Clone, PartialEq)]
pub enum WarehouseError {
    #[error("a transfer cannot start and end at the same warehouse")]
    NoSelfTransfer,
    #[error("invalid state transition: cannot {action} from '{from}'")]
    InvalidTransition { from: String, action: String },
    #[error("an incident requires a non-empty summary")]
    EmptySummary,
    #[error("break bulk requires at least one item")]
    EmptyBreakBulk,
    #[error("geofence radius must be positive (got {radius})")]
    InvalidGeofence { radius: f64 },
    #[error("incident is already resolved")]
    AlreadyResolved,
}

fn transfer_status_str(status: TransferStatus) -> String {
    match status {
        TransferStatus::Created => "created".into(),
        TransferStatus::Departed => "departed".into(),
        TransferStatus::Arrived => "arrived".into(),
        TransferStatus::Cancelled => "cancelled".into(),
    }
}

fn pallet_status_str(status: PalletStatus) -> String {
    match status {
        PalletStatus::Received => "received".into(),
        PalletStatus::InStorage => "in_storage".into(),
        PalletStatus::InTransit => "in_transit".into(),
        PalletStatus::BreakBulk => "break_bulk".into(),
        PalletStatus::Dispatched => "dispatched".into(),
        PalletStatus::Used => "used".into(),
    }
}

fn incident_status_str(status: IncidentStatus) -> String {
    match status {
        IncidentStatus::Open => "open".into(),
        IncidentStatus::Investigating => "investigating".into(),
        IncidentStatus::Resolved => "resolved".into(),
    }
}
