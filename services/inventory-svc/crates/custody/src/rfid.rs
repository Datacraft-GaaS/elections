use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::CustodyEvent;

/// Heartbeat emitted by an RFID/GPS reader attached to a kit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitHeartbeat {
    pub kit_id: String,
    pub rfid_tag: String,
    pub gps_lat: f64,
    pub gps_lon: f64,
    pub battery_pct: u8,
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
