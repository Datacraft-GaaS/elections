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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FindingStatus {
    #[default]
    Submitted,
    Reviewed,
    Approved,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FindingSeverity {
    High,
    #[default]
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubmitterCategory {
    CommissionDept,
    Iab,
    Keog,
    Elog,
    EuEom,
    DomesticObserver,
    Media,
    #[default]
    Citizen,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FindingType {
    #[default]
    Technical,
    Process,
    Legal,
    Security,
    Accessibility,
    Communication,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EelResponseType {
    #[default]
    Accepted,
    AcceptedWithModification,
    Declined,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Finding {
    pub id: String,
    pub election_cycle: String,
    pub submitter_category: SubmitterCategory,
    pub submitter_id: Option<String>,
    pub finding_type: FindingType,
    pub severity: FindingSeverity,
    pub description: String,
    pub evidence_refs: Option<String>,
    pub recommended_action: Option<String>,
    pub is_long_drip: bool,
    pub duplicate_of: Option<String>,
    pub status: FindingStatus,
    pub submitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Recommendation {
    pub id: String,
    pub finding_ids: Vec<String>,
    pub recommendation_text: String,
    pub eel_response_type: Option<EelResponseType>,
    pub eel_response_rationale: Option<String>,
    pub owner_dept_id: Option<String>,
    pub implementation_date: Option<String>,
    pub requires_legislation: bool,
    pub parliament_referral_id: Option<String>,
    pub chair_signed_at: Option<String>,
    pub chair_signed_by: Option<String>,
    pub status: String,
    pub next_cycle_handoff_at: Option<String>,
    pub last_long_drip_review: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParliamentReferral {
    pub id: String,
    pub recommendation_ids: Vec<String>,
    pub referral_package_hash: Option<String>,
    pub chair_signed_at: Option<String>,
    pub clerk_received_at: Option<String>,
    pub parliament_response: Option<String>,
    pub bill_reference: Option<String>,
    pub parliament_response_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LongDripAlert {
    pub id: String,
    pub finding_id: String,
    pub review_due_at: String,
    pub reviewer_id: Option<String>,
    pub reviewed_at: Option<String>,
    pub escalated_to_commissioner: bool,
    pub escalated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublicTrackerItem {
    pub recommendation_id: String,
    pub recommendation_text: String,
    pub eel_response_type: Option<EelResponseType>,
    pub status: String,
    pub chair_signed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitFindingRequest {
    pub election_cycle: String,
    pub submitter_category: SubmitterCategory,
    pub submitter_id: Option<String>,
    pub finding_type: FindingType,
    pub severity: FindingSeverity,
    pub description: String,
    pub evidence_refs: Option<String>,
    pub recommended_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TriageFindingRequest {
    pub status: FindingStatus,
    pub duplicate_of: Option<String>,
    pub is_long_drip: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignRecommendationRequest {
    pub signed_by: String,
    pub hsm_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateParliamentReferralRequest {
    pub recommendation_ids: Vec<String>,
    pub referral_package_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecordParliamentResponseRequest {
    pub parliament_response: String,
    pub bill_reference: Option<String>,
    pub parliament_response_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecordLongDripReviewRequest {
    pub reviewer_id: String,
    pub escalated_to_commissioner: bool,
}
