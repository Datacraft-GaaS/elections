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

//! Round-robin assignment with reviewer specialisation and workload balancing.
//!
//! Reviewers are routed by biometric modality match first, then by lowest
//! active case count, with round-robin tiebreaking.

use uuid::Uuid;

use crate::domain::{BiometricModality, Reviewer, ReviewerTier};

/// Select the best reviewer for a case from `pool`.
///
/// Selection order:
/// 1. Tier match (must match `required_tier`).
/// 2. Specialisation match (at least one modality overlap with `required_modalities`).
/// 3. Lowest `active_case_count` among specialised candidates.
/// 4. Round-robin (lowest index in the pool) as tiebreaker.
///
/// Returns `None` if no reviewer in the pool matches the tier.
pub fn select_reviewer<'a>(
    pool: &'a [Reviewer],
    required_tier: &ReviewerTier,
    required_modalities: &[BiometricModality],
) -> Option<&'a Reviewer> {
    let tier_match: Vec<&Reviewer> = pool.iter().filter(|r| &r.tier == required_tier).collect();

    if tier_match.is_empty() {
        return None;
    }

    // Prefer specialisation overlap; fall back to any tier match.
    let specialised: Vec<&Reviewer> = tier_match
        .iter()
        .copied()
        .filter(|r| {
            required_modalities
                .iter()
                .any(|m| r.specialisations.contains(m))
        })
        .collect();

    let candidates = if specialised.is_empty() {
        &tier_match
    } else {
        &specialised
    };

    // Lowest active_case_count wins; first in list breaks tie.
    candidates
        .iter()
        .copied()
        .min_by_key(|r| r.active_case_count)
}

/// Reviewer assignment event: pairs a case id with a reviewer id.
#[derive(Debug, Clone)]
pub struct Assignment {
    pub case_id: Uuid,
    pub reviewer_id: Uuid,
}
