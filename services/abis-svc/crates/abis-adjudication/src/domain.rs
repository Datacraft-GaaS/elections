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

//! Core domain types for adjudication workflow.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Priority ordering for adjudication cases. Higher priority jumps the queue.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CasePriority {
    Standard,
    Urgent,
    ElectionDayHold,
}

/// Reviewer tier; determines SLA and quorum requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewerTier {
    Tier1Analyst,
    Tier2Supervisor,
    Tier3Panel,
}

/// Life-cycle states of an adjudication case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseState {
    Open,
    Assigned,
    InReview,
    Decided,
    Escalated,
    Withdrawn,
    Reopened,
}

/// Outcome recorded when a case is decided.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjudicationDecision {
    ConfirmMatch(Uuid),
    ConfirmNoMatch,
    MergeInto(Uuid),
    SplitFrom(Uuid),
    Withdraw(String),
    Escalate(String),
}

/// SLA policy per tier (hours).
pub struct AdjudicationSlaPolicy;

impl AdjudicationSlaPolicy {
    pub const TIER1_HOURS: i64 = 4;
    pub const TIER2_HOURS: i64 = 12;
    pub const TIER3_HOURS: i64 = 48;

    /// Returns SLA hours for the given tier.
    pub fn hours_for(tier: &ReviewerTier) -> i64 {
        match tier {
            ReviewerTier::Tier1Analyst => Self::TIER1_HOURS,
            ReviewerTier::Tier2Supervisor => Self::TIER2_HOURS,
            ReviewerTier::Tier3Panel => Self::TIER3_HOURS,
        }
    }

    /// Returns the SLA deadline from a given open time.
    pub fn deadline_from(opened_at: DateTime<Utc>, tier: &ReviewerTier) -> DateTime<Utc> {
        opened_at + chrono::Duration::hours(Self::hours_for(tier))
    }

    /// Returns true if the deadline has passed.
    pub fn is_breached(deadline: DateTime<Utc>) -> bool {
        Utc::now() > deadline
    }
}

/// An adjudication case opened for an enrolment job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjudicationCase {
    pub id: Uuid,
    pub opened_at: DateTime<Utc>,
    pub priority: CasePriority,
    pub tier: ReviewerTier,
    pub state: CaseState,
    pub enrolment_job_id: Option<Uuid>,
    pub probable_match_subject_ids: Vec<Uuid>,
    pub evidence_bundle_ref: Uuid,
    pub assigned_to: Option<Uuid>,
    pub assigned_at: Option<DateTime<Utc>>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision: Option<AdjudicationDecision>,
    pub sla_deadline_at: DateTime<Utc>,
}

impl AdjudicationCase {
    /// Construct a new Open case with SLA set from tier and priority.
    pub fn new(
        enrolment_job_id: Option<Uuid>,
        probable_match_subject_ids: Vec<Uuid>,
        evidence_bundle_ref: Uuid,
        priority: CasePriority,
        tier: ReviewerTier,
    ) -> Self {
        let now = Utc::now();
        let sla_deadline_at = AdjudicationSlaPolicy::deadline_from(now, &tier);
        Self {
            id: Uuid::now_v7(),
            opened_at: now,
            priority,
            tier,
            state: CaseState::Open,
            enrolment_job_id,
            probable_match_subject_ids,
            evidence_bundle_ref,
            assigned_to: None,
            assigned_at: None,
            decided_at: None,
            decision: None,
            sla_deadline_at,
        }
    }

    /// Returns true if SLA has been breached.
    pub fn is_sla_breached(&self) -> bool {
        AdjudicationSlaPolicy::is_breached(self.sla_deadline_at)
    }

    /// Returns remaining SLA hours (negative = breached).
    pub fn sla_hours_remaining(&self) -> i64 {
        let remaining = self.sla_deadline_at.signed_duration_since(Utc::now());
        remaining.num_hours()
    }
}

/// A single reviewer action on a case (append-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerAction {
    pub id: Uuid,
    pub case_id: Uuid,
    pub reviewer_id: Uuid,
    pub action_type: ActionType,
    pub comment: Option<String>,
    pub evidence_added: Vec<Uuid>,
    pub taken_at: DateTime<Utc>,
    /// Ed25519 signature over canonical_json.
    pub hsm_signature: Vec<u8>,
    /// Hex-encoded Ed25519 verifying key.
    pub public_key_hex: String,
    /// BLAKE3 hash of the previous action in the chain (None for first action).
    pub prev_action_hash: Option<String>,
    /// RFC 8785 canonical JSON of the action (excludes signature fields).
    pub canonical_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Assign,
    StartReview,
    Decide,
    Escalate,
    AddEvidence,
    Withdraw,
    Reopen,
    PanelVote,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionType::Assign => "Assign",
            ActionType::StartReview => "StartReview",
            ActionType::Decide => "Decide",
            ActionType::Escalate => "Escalate",
            ActionType::AddEvidence => "AddEvidence",
            ActionType::Withdraw => "Withdraw",
            ActionType::Reopen => "Reopen",
            ActionType::PanelVote => "PanelVote",
        }
    }
}

/// A single panel member vote for Tier3 cases (M-of-N approval).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelVote {
    pub id: Uuid,
    pub case_id: Uuid,
    pub reviewer_id: Uuid,
    pub vote: Vote,
    pub rationale: Option<String>,
    pub voted_at: DateTime<Utc>,
    pub hsm_signature: Vec<u8>,
    pub public_key_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vote {
    Approve,
    Reject,
    Abstain,
}

/// Reviewer profile for workload balancing and specialisation routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reviewer {
    pub id: Uuid,
    pub tier: ReviewerTier,
    pub specialisations: Vec<BiometricModality>,
    pub active_case_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiometricModality {
    Fingerprint,
    Face,
    Iris,
}
