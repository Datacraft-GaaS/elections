//! Public contract models (governance-by-election/operations.yaml).

use serde::{Deserialize, Serialize};

/// Request body for declaring a by-election.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByElectionDeclaration {
    pub trigger_reason: String,
    pub constituency_code: String,
    pub gazette_ref: String,
    pub election_date: String,
}

/// Public by-election view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByElection {
    pub by_election_id: String,
    pub trigger_reason: String,
    pub constituency_code: String,
    pub gazette_ref: String,
    pub election_date: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for a status transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusTransition {
    pub new_status: String,
    pub reason: String,
}
