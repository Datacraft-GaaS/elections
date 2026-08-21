//! Event-type constants and payload structs for M25 special voter workflows.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const EVENT_SPECIAL_FACILITY_OPENED: &str = "eel.support.special_facility_opened.v1";
pub const EVENT_SPECIAL_VOTER_EVENT: &str = "eel.support.special_voter_event.v1";
pub const EVENT_SPECIAL_FORM34_TRANSMITTED: &str = "eel.support.special_form34_transmitted.v1";
pub const EVENT_SECRECY_VIOLATION_DETECTED: &str = "eel.support.secrecy_violation_detected.v1";
pub const EVENT_IDP_REASSIGNMENT_RECORDED: &str = "eel.support.idp_reassignment_recorded.v1";
pub const EVENT_S44A_OVERRIDE_REQUESTED: &str = "eel.support.s44a_override_requested.v1";
pub const EVENT_S44A_OVERRIDE_APPROVED: &str = "eel.support.s44a_override_approved.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialFacilityOpenedPayload {
    pub facility_id: Uuid,
    pub category: String,
    pub county_code: String,
    pub kit_ids: Vec<String>,
    pub presiding_officer_id: String,
    pub voting_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialVoterEventPayload {
    pub event_id: Uuid,
    pub facility_id: Uuid,
    pub category: String,
    pub event_type: String,
    pub officer_id: String,
    pub witness_ids: Vec<String>,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialForm34TransmittedPayload {
    pub form_id: Uuid,
    pub facility_id: Uuid,
    pub form_variant: String,
    pub transmitted_at: String,
    pub merkle_node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecrecyViolationPayload {
    pub facility_id: Uuid,
    pub detected_at: String,
    pub officer_id: String,
    pub proximity_meters: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDPReassignmentPayload {
    pub reassignment_id: Uuid,
    pub original_station_code: String,
    pub idp_facility_id: Uuid,
    pub displacement_source: String,
    pub effective_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S44AOverridePayload {
    pub facility_id: Uuid,
    pub reason_code: String,
    pub requesting_officer_id: String,
    pub countersigning_officer_id: Option<String>,
    pub approved: bool,
}
