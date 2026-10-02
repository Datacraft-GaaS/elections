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

//! Adjudication case management.

use crate::domain::{AdjudicationCase, AdjudicationDecision, AdjudicationStatus, MatchResult};

pub fn open_adjudication_case(
    match_result: &MatchResult,
    voter_ids: Vec<String>,
) -> AdjudicationCase {
    AdjudicationCase {
        case_id: uuid::Uuid::new_v4().to_string(),
        match_result_id: match_result.result_id.clone(),
        voter_ids,
        status: AdjudicationStatus::Open,
        investigator_id: None,
        decision: None,
        reason_code: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}

pub fn record_decision(case: &mut AdjudicationCase, decision: AdjudicationDecision) {
    case.decision = Some(decision.decision.clone());
    case.reason_code = Some(decision.reason_code.clone());
    case.investigator_id = Some(decision.officer_id.clone());
    case.status = AdjudicationStatus::Resolved;
}

pub fn is_ambiguous(score: f32, lower: f32, upper: f32) -> bool {
    score >= lower && score < upper
}
