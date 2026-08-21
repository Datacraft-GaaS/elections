use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSnapshot {
    pub station_code: String,
    pub kit_throughput_per_hour: u32,
    pub clerk_report_count: u32,
    pub cv_headcount: u32,
    pub qr_scan_count: u32,
    pub estimated_wait_minutes: u32,
    pub disenfranchisement_flag: bool,
    pub s14a_recommended: bool,
    pub fused_at: DateTime<Utc>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSnapshotEvent {
    pub event_type: String,
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub snapshot: QueueSnapshot,
}

impl QueueSnapshotEvent {
    pub fn new(snapshot: QueueSnapshot) -> Self {
        Self {
            event_type: "eel.queue.snapshot.v1".to_string(),
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            snapshot,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitTimeBreachedEvent {
    pub event_type: String,
    pub station_code: String,
    pub wait_minutes: u32,
    pub threshold_minutes: u32,
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisenfranchisementFlagEvent {
    pub event_type: String,
    pub station_code: String,
    pub reason: String,
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S14aRecommendedEvent {
    pub event_type: String,
    pub station_code: String,
    pub recommendation_id: Uuid,
    pub hsm_pending: bool,
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureFusionInput {
    pub kit_throughput_per_hour: u32,
    pub clerk_report_count: u32,
    pub cv_headcount: u32,
    pub qr_scan_count: u32,
}
