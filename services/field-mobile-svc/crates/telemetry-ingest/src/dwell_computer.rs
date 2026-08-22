//! Nightly job that computes geo dwells from location signals
//! and writes rows to eel_telemetry_geo_dwells.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::TelemetryGeoDwell;

/// A location ping used as input to dwell computation.
#[derive(Debug, Clone)]
pub struct LocationPing {
    pub device_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub polygon_ref: String,
}

/// Compute geo-dwell periods from an ordered sequence of location pings.
///
/// A dwell is a contiguous run of pings within the same `polygon_ref`.
/// `exited_at` is set to the first ping in the next polygon (or `None` if
/// the device was still inside at the end of the window).
pub fn compute_dwells(pings: &[LocationPing]) -> Vec<TelemetryGeoDwell> {
    if pings.is_empty() {
        return vec![];
    }

    let mut dwells: Vec<TelemetryGeoDwell> = Vec::new();
    let mut run_start: &LocationPing = &pings[0];
    let mut current_polygon = &pings[0].polygon_ref;

    for ping in pings.iter().skip(1) {
        if &ping.polygon_ref != current_polygon {
            let entered_at = run_start.occurred_at;
            let exited_at = ping.occurred_at;
            let dwell_seconds = (exited_at - entered_at).num_seconds() as i32;

            dwells.push(TelemetryGeoDwell {
                dwell_id: Uuid::now_v7(),
                device_id: run_start.device_id,
                polygon_ref: current_polygon.clone(),
                entered_at,
                exited_at: Some(exited_at),
                dwell_seconds: Some(dwell_seconds),
            });

            run_start = ping;
            current_polygon = &ping.polygon_ref;
        }
    }

    // Last open dwell (device still inside at end of window).
    dwells.push(TelemetryGeoDwell {
        dwell_id: Uuid::now_v7(),
        device_id: run_start.device_id,
        polygon_ref: current_polygon.clone(),
        entered_at: run_start.occurred_at,
        exited_at: None,
        dwell_seconds: None,
    });

    dwells
}
