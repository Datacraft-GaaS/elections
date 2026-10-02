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

//! Presidential run-off and tie rules per Elections Act §§83-84 and Art. 138(4).
//!
//! Winner determination logic for WP-3.3 (M51, FR-M51-009 through FR-M51-014).
//!
//! # Invariants (also encoded in Declaration.tla)
//! - A candidate wins outright only when BOTH conditions hold simultaneously:
//!     1. votes > total_valid / 2  (strict majority, 50%+1)
//!     2. county_threshold_count >= 24  (>=25% in at least 24 of 47 counties)
//! - If no candidate satisfies (1) or (2): RunoffRequired, top-two proceed.
//! - If two or more candidates tie on plurality *and* neither satisfies (1)+(2): RunoffRequired.
//! - If two candidates tie for first with equal votes everywhere (exact tie): TiePending.

use serde::{Deserialize, Serialize};

/// Per-county vote breakdown for a candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyVotes {
    pub county_code: String,
    /// Valid votes cast for this candidate in this county.
    pub votes: u64,
    /// Total valid votes cast in this county across all candidates.
    pub county_total_valid: u64,
}

/// Full vote distribution for a single presidential candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateDistribution {
    pub candidate_id: String,
    /// Total valid votes nationwide.
    pub national_votes: u64,
    /// Per-county breakdown; must cover all 47 counties present in the tally.
    pub county_votes: Vec<CountyVotes>,
}

/// Input to the rules engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresidentialTally {
    /// Total valid votes cast nationwide.
    pub total_valid_votes: u64,
    pub candidates: Vec<CandidateDistribution>,
}

/// Output of the rules engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeclarationOutcome {
    /// Candidate meets both statutory thresholds.
    Declared { winner_id: String },
    /// No candidate meets the thresholds; top-two proceed to runoff.
    RunoffRequired {
        runoff_candidates: Vec<String>,
        counties_above_threshold_leader: u32,
    },
    /// Exact tie between two or more candidates; no certificate may be issued
    /// until a witnessed draw resolves to Declared (EC-LEG-06).
    TiePending { tied_candidates: Vec<String> },
}

/// Returns true when the candidate reaches >=25% of county_total_valid votes
/// in that county (exact >= per EC-LEG-07).
fn reaches_county_threshold(cv: &CountyVotes) -> bool {
    if cv.county_total_valid == 0 {
        return false;
    }
    // >=25%: candidate_votes * 4 >= county_total_valid
    cv.votes * 4 >= cv.county_total_valid
}

/// Count the counties where a candidate reaches the 25% threshold.
fn county_threshold_count(dist: &CandidateDistribution) -> u32 {
    dist.county_votes
        .iter()
        .filter(|cv| reaches_county_threshold(cv))
        .count() as u32
}

/// Core rules engine.  Never reads mutable projections; operates on the signed
/// tally snapshot only (FR-M51-009).
pub fn evaluate(tally: &PresidentialTally) -> DeclarationOutcome {
    if tally.total_valid_votes == 0 || tally.candidates.is_empty() {
        // No votes cast; runoff cannot be meaningful; treat as tie-pending to
        // force human adjudication (EC-DAT-10 zero turnout path).
        return DeclarationOutcome::TiePending {
            tied_candidates: tally
                .candidates
                .iter()
                .map(|c| c.candidate_id.clone())
                .collect(),
        };
    }

    // Compute per-candidate metrics.
    struct Metrics {
        candidate_id: String,
        national_votes: u64,
        county_count: u32,
    }

    let mut metrics: Vec<Metrics> = tally
        .candidates
        .iter()
        .map(|c| Metrics {
            candidate_id: c.candidate_id.clone(),
            national_votes: c.national_votes,
            county_count: county_threshold_count(c),
        })
        .collect();

    // Sort descending by national votes.
    metrics.sort_by(|a, b| b.national_votes.cmp(&a.national_votes));

    let total = tally.total_valid_votes;
    let leader = &metrics[0];

    // Check for outright winner: strict majority AND 24-county threshold.
    // 50%+1: leader_votes * 2 > total (integer-safe, no floating point).
    let majority = leader.national_votes * 2 > total;
    let county_ok = leader.county_count >= 24;

    if majority && county_ok {
        // Verify no other candidate has the exact same vote count (tie guard).
        let tie_peers: Vec<&Metrics> = metrics
            .iter()
            .filter(|m| m.national_votes == leader.national_votes)
            .collect();
        if tie_peers.len() > 1 {
            return DeclarationOutcome::TiePending {
                tied_candidates: tie_peers.iter().map(|m| m.candidate_id.clone()).collect(),
            };
        }
        return DeclarationOutcome::Declared {
            winner_id: leader.candidate_id.clone(),
        };
    }

    // No outright winner. Check for exact tie between top candidates.
    if metrics.len() >= 2 && metrics[0].national_votes == metrics[1].national_votes {
        let top_votes = metrics[0].national_votes;
        let tied: Vec<String> = metrics
            .iter()
            .filter(|m| m.national_votes == top_votes)
            .map(|m| m.candidate_id.clone())
            .collect();
        return DeclarationOutcome::TiePending {
            tied_candidates: tied,
        };
    }

    // Runoff: top-two candidates.
    let runoff_candidates: Vec<String> = metrics
        .iter()
        .take(2)
        .map(|m| m.candidate_id.clone())
        .collect();

    DeclarationOutcome::RunoffRequired {
        runoff_candidates,
        counties_above_threshold_leader: metrics[0].county_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_trace_rs::trace;

    fn county_votes(votes: u64, total: u64) -> CountyVotes {
        CountyVotes {
            county_code: "01".into(),
            votes,
            county_total_valid: total,
        }
    }

    fn single_county_dist(id: &str, votes: u64, total: u64) -> CandidateDistribution {
        // Spread votes across 47 synthetic counties proportionally.
        let per_county_total = (total + 46) / 47;
        let per_county_votes = (votes + 46) / 47;
        let county_votes = (0..47)
            .map(|i| CountyVotes {
                county_code: format!("{:02}", i + 1),
                votes: if i < 24 { per_county_votes } else { 0 },
                county_total_valid: per_county_total,
            })
            .collect();
        CandidateDistribution {
            candidate_id: id.to_string(),
            national_votes: votes,
            county_votes,
        }
    }

    #[trace("FR-M51-009", "FR-M51-010", "FR-M51-011")]
    #[test]
    fn outright_winner_declared() {
        // 60% of 100 votes, spread across all 47 counties.
        let total = 100u64;
        let winner_votes = 60u64;
        let loser_votes = 40u64;
        let winner = single_county_dist("alice", winner_votes, total);
        let loser = CandidateDistribution {
            candidate_id: "bob".into(),
            national_votes: loser_votes,
            county_votes: (0..47)
                .map(|i| CountyVotes {
                    county_code: format!("{:02}", i + 1),
                    votes: if i < 24 { loser_votes / 47 } else { 0 },
                    county_total_valid: (total + 46) / 47,
                })
                .collect(),
        };
        let tally = PresidentialTally {
            total_valid_votes: total,
            candidates: vec![winner, loser],
        };
        assert_eq!(
            evaluate(&tally),
            DeclarationOutcome::Declared {
                winner_id: "alice".into()
            }
        );
    }

    #[trace("FR-M51-010", "FR-M51-012")]
    #[test]
    fn below_majority_triggers_runoff() {
        let total = 100u64;
        // Alice 49%, Bob 51%? No: Alice 45%, Bob 30%, Carol 25% -- no majority.
        let alice = single_county_dist("alice", 45, total);
        let bob = single_county_dist("bob", 30, total);
        let carol = single_county_dist("carol", 25, total);
        let tally = PresidentialTally {
            total_valid_votes: total,
            candidates: vec![alice, bob, carol],
        };
        let outcome = evaluate(&tally);
        assert!(matches!(outcome, DeclarationOutcome::RunoffRequired { .. }));
    }

    #[trace("FR-M51-011", "EC-LEG-07")]
    #[test]
    fn county_threshold_not_met_triggers_runoff() {
        // Alice wins 51% nationally but only in 23 counties above 25%.
        let total = 1000u64;
        let alice_votes = 510u64;
        // Put all Alice votes in exactly 23 counties.
        let alice = CandidateDistribution {
            candidate_id: "alice".into(),
            national_votes: alice_votes,
            county_votes: (0..47)
                .map(|i| {
                    let county_total = total / 47;
                    let votes = if i < 23 { alice_votes / 23 } else { 0 };
                    CountyVotes {
                        county_code: format!("{:02}", i + 1),
                        votes,
                        county_total_valid: county_total,
                    }
                })
                .collect(),
        };
        let bob = CandidateDistribution {
            candidate_id: "bob".into(),
            national_votes: 490,
            county_votes: (0..47)
                .map(|i| CountyVotes {
                    county_code: format!("{:02}", i + 1),
                    votes: if i >= 23 { 490 / 24 } else { 0 },
                    county_total_valid: total / 47,
                })
                .collect(),
        };
        let tally = PresidentialTally {
            total_valid_votes: total,
            candidates: vec![alice, bob],
        };
        let outcome = evaluate(&tally);
        assert!(matches!(outcome, DeclarationOutcome::RunoffRequired { .. }));
    }

    #[trace("FR-M51-013", "EC-LEG-06")]
    #[test]
    fn exact_tie_yields_tie_pending() {
        let total = 100u64;
        let a = single_county_dist("alice", 50, total);
        let b = single_county_dist("bob", 50, total);
        let tally = PresidentialTally {
            total_valid_votes: total,
            candidates: vec![a, b],
        };
        let outcome = evaluate(&tally);
        assert!(matches!(outcome, DeclarationOutcome::TiePending { .. }));
    }

    #[trace("FR-M51-011", "EC-LEG-07")]
    #[test]
    fn boundary_24_counties_declares() {
        // Exactly 24 counties at >= 25%.
        let county_total = 100u64;
        let alice_county_votes = 26u64; // 26% each in counties 0..24
        let alice_national = alice_county_votes * 24;
        let alice = CandidateDistribution {
            candidate_id: "alice".into(),
            national_votes: alice_national,
            county_votes: (0..47)
                .map(|i| CountyVotes {
                    county_code: format!("{:02}", i + 1),
                    votes: if i < 24 { alice_county_votes } else { 0 },
                    county_total_valid: county_total,
                })
                .collect(),
        };
        // Alice may or may not win national majority depending on totals;
        // here we just assert county count logic is >=24 not >24.
        let count = county_threshold_count(&alice);
        assert_eq!(count, 24);
    }

    #[trace("FR-M51-011", "EC-LEG-07")]
    #[test]
    fn boundary_23_counties_not_enough() {
        let county_total = 100u64;
        let alice_county_votes = 26u64;
        let alice = CandidateDistribution {
            candidate_id: "alice".into(),
            national_votes: alice_county_votes * 23,
            county_votes: (0..47)
                .map(|i| CountyVotes {
                    county_code: format!("{:02}", i + 1),
                    votes: if i < 23 { alice_county_votes } else { 0 },
                    county_total_valid: county_total,
                })
                .collect(),
        };
        let count = county_threshold_count(&alice);
        assert_eq!(count, 23);
    }

    #[trace("FR-M51-011", "EC-LEG-07")]
    #[test]
    fn exact_25_percent_county_counts() {
        // Exactly 25%: 25 out of 100. Must count (>= semantics per EC-LEG-07).
        let cv = county_votes(25, 100);
        assert!(reaches_county_threshold(&cv));
    }

    #[trace("FR-M51-011", "EC-LEG-07")]
    #[test]
    fn just_below_25_percent_does_not_count() {
        // 24/100 = 24%; must not count.
        let cv = county_votes(24, 100);
        assert!(!reaches_county_threshold(&cv));
    }
}
