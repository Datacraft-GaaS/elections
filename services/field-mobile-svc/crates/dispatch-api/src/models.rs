use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerAssignment {
    pub id: Uuid,
    pub worker_id: Uuid,
    pub station_code: String,
    pub election_id: Uuid,
    pub role: String,
    pub report_time: DateTime<Utc>,
    pub supervisor_id: Uuid,
    pub status: AssignmentStatus,
    pub accepted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssignmentStatus {
    Pending,
    Accepted,
    Declined,
    Reassigned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerCheckIn {
    pub id: Uuid,
    pub worker_id: Uuid,
    pub station_code: String,
    pub election_id: Uuid,
    pub biometric_hash: String,
    pub gps_lat: f64,
    pub gps_lon: f64,
    pub gps_accuracy_m: f32,
    pub check_in_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerStatusPing {
    pub id: Uuid,
    pub worker_id: Uuid,
    pub station_code: String,
    pub status: WorkerStatus,
    pub reason: Option<String>,
    pub pinged_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkerStatus {
    OnStation,
    Break,
    Absent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerCheckOut {
    pub id: Uuid,
    pub worker_id: Uuid,
    pub station_code: String,
    pub election_id: Uuid,
    pub gps_lat: f64,
    pub gps_lon: f64,
    pub check_out_at: DateTime<Utc>,
    pub hours_worked: f32,
    pub payroll_trigger_id: Option<Uuid>,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckInRequest {
    pub worker_id: Uuid,
    pub station_code: String,
    pub election_id: Uuid,
    pub biometric_hash: String,
    pub gps_lat: f64,
    pub gps_lon: f64,
    pub gps_accuracy_m: f32,
    pub signature: String,
}
