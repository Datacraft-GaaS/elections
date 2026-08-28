//! Public contract models (gazette/calendar.yaml inspection surface).

use serde::{Deserialize, Serialize};

/// Public inspection-window view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionWindow {
    pub window_id: String,
    pub opens_at: String,
    pub closes_at: String,
    pub status: String,
    pub venues: Vec<String>,
}

/// Public list-objection view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListObjection {
    pub objection_id: String,
    pub reference_code: String,
    pub status: String,
    pub submitted_at: String,
}

/// Public objection status view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectionStatus {
    pub reference_code: String,
    pub status: String,
    pub updated_at: String,
}
