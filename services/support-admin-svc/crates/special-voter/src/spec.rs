//! Typed models for `support-special-voter/workflows.yaml`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mirrors `SpecialVoterFacility`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Facility {
    pub id: Uuid,
    pub category: String,
    pub county_code: String,
    pub presiding_officer_id: String,
    pub voting_date: String,
    pub published_voter_count: Option<i32>,
    pub session_state: String,
    pub violation_pending_ack: Option<bool>,
}

/// Mirrors `SpecialVoterEventRequest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRequest {
    pub event_type: String,
    pub officer_id: String,
    pub witness_ids: Vec<String>,
    pub reason_code: Option<String>,
}

/// Mirrors `SpecialVoterEventRecord`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub event_type: String,
    pub officer_id: String,
    pub witness_ids: Vec<String>,
    pub reason_code: Option<String>,
    pub merkle_node_id: Option<String>,
    pub event_at: DateTime<Utc>,
}

/// Mirrors `SpecialForm34Request`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form34Request {
    pub facility_id: Uuid,
    pub form_variant: String,
    pub tallies: serde_json::Value,
    pub signatures: Vec<String>,
}

/// Mirrors `IDPReassignmentRequest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdpReassignmentRequest {
    pub voter_id_enc: String,
    pub original_station_code: String,
    pub idp_facility_id: Uuid,
    pub displacement_source: String,
    pub effective_date: String,
}
