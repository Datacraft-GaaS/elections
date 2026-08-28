//! Vehicle transport and security-deployment domain (M20 inventory).
//!
//! Two closely-related concerns for moving ballot lots from warehouse to
//! polling station:
//!
//! 1. **Transport** — a `Vehicle` carries a `VehicleLoad` of ballot lots
//!    (`LoadItem`s), each captured by an RFID or barcode scan, with capacity
//!    enforced per vehicle.
//! 2. **Security** — `SecurityAgent`s are assigned to escort a vehicle or
//!    guard a polling station within a time window, with no double-booking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleType {
    #[default]
    Truck,
    Van,
    Air,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleStatus {
    #[default]
    Available,
    Loading,
    Dispatched,
    Returned,
}

/// How a ballot lot was identified at load time. Lots may carry an RFID tag
/// (13.56 MHz HF, ISO 14443A/15693) or a Code 128 barcode (ISO/IEC 15417).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanMethod {
    #[default]
    Rfid,
    Barcode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadStatus {
    #[default]
    Loading,
    Dispatched,
    Delivered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentTarget {
    #[default]
    Vehicle,
    Station,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentStatus {
    #[default]
    Assigned,
    OnDuty,
    Completed,
}

// ---------------------------------------------------------------------------
// Transport
// ---------------------------------------------------------------------------

/// A transport vehicle (truck, van, or air-freight) with a ballot capacity
/// measured in ballot-box-equivalent units.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: String,
    pub registration: String,
    pub vehicle_type: VehicleType,
    /// Maximum ballot-box-equivalent units the vehicle can carry.
    pub capacity: u64,
    pub status: VehicleStatus,
}

impl Vehicle {
    /// A vehicle can be loaded while available or already partway loaded.
    pub fn can_be_loaded(&self) -> bool {
        matches!(
            self.status,
            VehicleStatus::Available | VehicleStatus::Loading
        )
    }
}

/// A single scanned ballot lot loaded onto a vehicle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadItem {
    pub id: String,
    pub lot_id: String,
    pub quantity: u64,
    pub scan_method: ScanMethod,
    /// The value read off the lot: RFID EPC or barcode digits.
    pub scan_value: String,
    pub scanned_by: String,
    pub scanned_at: DateTime<Utc>,
}

/// A dispatch load: the set of ballot lots loaded onto a vehicle for one
/// destination, optionally escorted by a security agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleLoad {
    pub id: String,
    pub vehicle_id: String,
    pub destination: String,
    pub escort_agent_id: Option<String>,
    pub status: LoadStatus,
    pub items: Vec<LoadItem>,
    pub loaded_at: DateTime<Utc>,
    pub dispatched_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
}

impl VehicleLoad {
    pub fn new(
        id: String,
        vehicle_id: String,
        destination: String,
        escort_agent_id: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            vehicle_id,
            destination,
            escort_agent_id,
            status: LoadStatus::Loading,
            items: Vec::new(),
            loaded_at: now,
            dispatched_at: None,
            delivered_at: None,
        }
    }

    pub fn loaded_quantity(&self) -> u64 {
        self.items.iter().map(|i| i.quantity).sum()
    }

    /// Scan and load a ballot lot. Enforces the scan capture, rejects
    /// duplicate lots, and respects the vehicle capacity.
    pub fn add_item(&mut self, vehicle: &Vehicle, item: LoadItem) -> Result<(), TransportError> {
        if self.status != LoadStatus::Loading {
            return Err(TransportError::NotLoading(status_str(self.status)));
        }
        if item.scan_value.trim().is_empty() || item.scanned_by.trim().is_empty() {
            return Err(TransportError::InvalidScan);
        }
        if self.items.iter().any(|i| i.lot_id == item.lot_id) {
            return Err(TransportError::LotAlreadyLoaded(item.lot_id.clone()));
        }
        let projected = self.loaded_quantity() + item.quantity;
        if projected > vehicle.capacity {
            return Err(TransportError::CapacityExceeded {
                requested: projected,
                capacity: vehicle.capacity,
            });
        }
        self.items.push(item);
        Ok(())
    }

    /// Mark the load dispatched. Requires at least one scanned item.
    pub fn dispatch(&mut self, now: DateTime<Utc>) -> Result<(), TransportError> {
        if self.status != LoadStatus::Loading {
            return Err(TransportError::NotLoading(status_str(self.status)));
        }
        if self.items.is_empty() {
            return Err(TransportError::EmptyLoad);
        }
        self.status = LoadStatus::Dispatched;
        self.dispatched_at = Some(now);
        Ok(())
    }

    /// Mark the load delivered at its destination.
    pub fn deliver(&mut self, now: DateTime<Utc>) -> Result<(), TransportError> {
        if self.status != LoadStatus::Dispatched {
            return Err(TransportError::NotLoading(status_str(self.status)));
        }
        self.status = LoadStatus::Delivered;
        self.delivered_at = Some(now);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Security deployment
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAgent {
    pub id: String,
    pub badge_number: String,
    pub unit: String,
}

/// A security agent assigned to escort a vehicle or guard a polling station
/// within a time window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAssignment {
    pub id: String,
    pub agent_id: String,
    pub target: AssignmentTarget,
    pub target_id: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub status: AssignmentStatus,
}

impl SecurityAssignment {
    pub fn new(
        id: String,
        agent_id: String,
        target: AssignmentTarget,
        target_id: String,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) -> Result<Self, TransportError> {
        if ends_at <= starts_at {
            return Err(TransportError::InvalidWindow);
        }
        Ok(Self {
            id,
            agent_id,
            target,
            target_id,
            starts_at,
            ends_at,
            status: AssignmentStatus::Assigned,
        })
    }

    /// True when `other` concerns the same agent and the two time windows
    /// overlap (non-empty intersection).
    pub fn overlaps(&self, other: &SecurityAssignment) -> bool {
        self.agent_id == other.agent_id
            && self.starts_at < other.ends_at
            && other.starts_at < self.ends_at
    }
}

/// Assign a security agent to a target, rejecting double-booking.
pub fn assign_security(
    id: String,
    agent_id: String,
    target: AssignmentTarget,
    target_id: String,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    existing: &[SecurityAssignment],
) -> Result<SecurityAssignment, TransportError> {
    let assignment = SecurityAssignment::new(
        id,
        agent_id.clone(),
        target,
        target_id.clone(),
        starts_at,
        ends_at,
    )?;
    if existing.iter().any(|e| assignment.overlaps(e)) {
        return Err(TransportError::AlreadyAssigned {
            agent: agent_id,
            target: target_id,
        });
    }
    Ok(assignment)
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TransportError {
    #[error("vehicle capacity exceeded: loading {requested} would exceed capacity {capacity}")]
    CapacityExceeded { requested: u64, capacity: u64 },
    #[error("load is not in 'loading' state (current: {0})")]
    NotLoading(String),
    #[error("cannot dispatch an empty load")]
    EmptyLoad,
    #[error("a lot {0} is already loaded; merge quantities or remove it first")]
    LotAlreadyLoaded(String),
    #[error("a scan requires a non-empty scan value and a scanning officer")]
    InvalidScan,
    #[error("assignment end must be after its start")]
    InvalidWindow,
    #[error("agent {agent} is already assigned to {target} in this window")]
    AlreadyAssigned { agent: String, target: String },
}

fn status_str(status: LoadStatus) -> String {
    match status {
        LoadStatus::Loading => "loading".into(),
        LoadStatus::Dispatched => "dispatched".into(),
        LoadStatus::Delivered => "delivered".into(),
    }
}
