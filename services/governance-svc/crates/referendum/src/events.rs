// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{Deserialize, Serialize};

/// eel.governance.referendum_configured.v1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferendumConfiguredV1 {
    pub event_type: String,
    pub referendum_id: String,
    pub election_id: String,
    pub county_threshold: u8,
    pub scheduled_date: String,
    pub occurred_at: String,
}

impl ReferendumConfiguredV1 {
    pub fn new(referendum_id: String, election_id: String, scheduled_date: String) -> Self {
        Self {
            event_type: "eel.governance.referendum_configured.v1".to_string(),
            referendum_id,
            election_id,
            county_threshold: crate::threshold::REQUIRED_COUNTY_THRESHOLD,
            scheduled_date,
            occurred_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// eel.governance.kit_ota_pushed.v1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitOtaPushedV1 {
    pub event_type: String,
    pub referendum_id: String,
    pub bundle_id: String,
    pub signed_bundle_hash: String,
    pub total_kits: u32,
    pub occurred_at: String,
}

impl KitOtaPushedV1 {
    pub fn new(
        referendum_id: String,
        bundle_id: String,
        signed_bundle_hash: String,
        total_kits: u32,
    ) -> Self {
        Self {
            event_type: "eel.governance.kit_ota_pushed.v1".to_string(),
            referendum_id,
            bundle_id,
            signed_bundle_hash,
            total_kits,
            occurred_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// eel.governance.county_confirmed.v1 -- emitted when a county result is certified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyConfirmedV1 {
    pub event_type: String,
    pub referendum_id: String,
    pub county_code: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub county_yes_majority: bool,
    pub counties_yes_so_far: u8,
    pub occurred_at: String,
}

impl CountyConfirmedV1 {
    pub fn new(
        referendum_id: String,
        county_code: String,
        yes_votes: u64,
        no_votes: u64,
        county_yes_majority: bool,
        counties_yes_so_far: u8,
    ) -> Self {
        Self {
            event_type: "eel.governance.county_confirmed.v1".to_string(),
            referendum_id,
            county_code,
            yes_votes,
            no_votes,
            county_yes_majority,
            counties_yes_so_far,
            occurred_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// eel.governance.national_declared.v1 -- emitted after both thresholds are confirmed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationalDeclaredV1 {
    pub event_type: String,
    pub referendum_id: String,
    pub total_yes: u64,
    pub total_no: u64,
    pub counties_yes_majority: u8,
    pub meets_county_threshold: bool,
    pub meets_national_majority: bool,
    /// PASSED | FAILED
    pub outcome: String,
    pub occurred_at: String,
}

impl NationalDeclaredV1 {
    pub fn new(
        referendum_id: String,
        total_yes: u64,
        total_no: u64,
        counties_yes_majority: u8,
        meets_county_threshold: bool,
        meets_national_majority: bool,
    ) -> Self {
        let outcome = if meets_county_threshold && meets_national_majority {
            "PASSED"
        } else {
            "FAILED"
        }
        .to_string();
        Self {
            event_type: "eel.governance.national_declared.v1".to_string(),
            referendum_id,
            total_yes,
            total_no,
            counties_yes_majority,
            meets_county_threshold,
            meets_national_majority,
            outcome,
            occurred_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
