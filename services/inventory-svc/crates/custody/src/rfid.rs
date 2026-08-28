use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::CustodyEvent;

/// Heartbeat emitted by an RFID/GPS reader attached to a kit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitHeartbeat {
    /// kit_id.
    pub kit_id: String,
    /// rfid_tag.
    pub rfid_tag: String,
    /// gps_lat.
    pub gps_lat: f64,
    /// gps_lon.
    pub gps_lon: f64,
    /// battery_pct.
    pub battery_pct: u8,
    /// ts.
    pub ts: DateTime<Utc>,
}

/// Converts an RFID heartbeat into a custody event for the asset-location log.
/// The event records the kit moving to its own location (self-reported position),
/// with an empty signature placeholder pending HSM countersigning.
pub fn process_heartbeat(hb: KitHeartbeat) -> CustodyEvent {
    let location = format!("{},{}", hb.gps_lat, hb.gps_lon);
    CustodyEvent {
        event_id: Uuid::now_v7().to_string(),
        asset_id: hb.kit_id.clone(),
        from_custodian: hb.kit_id.clone(),
        to_custodian: hb.kit_id.clone(),
        location,
        ts_hlc: hb.ts.timestamp_millis(),
        signature: String::new(),
        seq: 0,
    }
}
