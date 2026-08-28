//! Cross-service event contracts for the transport & security-deployment
//! domain (M20 inventory, design §8). Payloads are the wire contract for
//! downstream consumers (ECC, IAB, logistics, boundary).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const VEHICLE_REGISTERED: &str = "eel.transport.vehicle_registered.v1";
pub const BALLOTS_LOADED: &str = "eel.transport.ballots_loaded.v1";
pub const VEHICLE_DISPATCHED: &str = "eel.transport.vehicle_dispatched.v1";
pub const VEHICLE_DELIVERED: &str = "eel.transport.vehicle_delivered.v1";
pub const SECURITY_ASSIGNED: &str = "eel.transport.security_assigned.v1";
pub const SECURITY_RELEASED: &str = "eel.transport.security_released.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleRegistered {
    pub vehicle_id: String,
    pub registration: String,
    pub vehicle_type: String,
    pub capacity: u64,
}

/// A ballot lot was scanned (RFID or barcode) and loaded onto a vehicle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotsLoaded {
    pub load_id: String,
    pub vehicle_id: String,
    pub lot_id: String,
    pub quantity: u64,
    /// "rfid" | "barcode".
    pub scan_method: String,
    pub scanned_by: String,
    pub scanned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleDispatched {
    pub load_id: String,
    pub vehicle_id: String,
    pub destination: String,
    pub escort_agent_id: Option<String>,
    pub dispatched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleDelivered {
    pub load_id: String,
    pub vehicle_id: String,
    pub destination: String,
    pub delivered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAssigned {
    pub assignment_id: String,
    pub agent_id: String,
    /// "vehicle" | "station".
    pub target: String,
    pub target_id: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}
