use rts_domain::aggregates::form34c::Form34C;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub aggregator_id: String,
    pub candidate_id: String,
    pub expected: u64,
    pub actual: u64,
}

pub struct DriftDetector {
    tallies: HashMap<String, Form34C>,
}
impl DriftDetector {
    pub fn new() -> Self {
        Self {
            tallies: HashMap::new(),
        }
    }
    pub fn record_tally(&mut self, aggregator_id: &str, form34c: &Form34C) -> Option<DriftReport> {
        self.tallies
            .insert(aggregator_id.to_string(), form34c.clone());
        if self.tallies.len() < 2 {
            return None;
        }
        let reference = self.tallies.values().next()?.clone();
        for (agg_id, tally) in &self.tallies {
            for candidate in &tally.candidate_totals {
                let expected = reference
                    .candidate_totals
                    .iter()
                    .find(|c| c.candidate_id == candidate.candidate_id)
                    .map(|c| c.votes as u64)
                    .unwrap_or(0);
                let actual = candidate.votes as u64;
                if expected != actual {
                    return Some(DriftReport {
                        aggregator_id: agg_id.clone(),
                        candidate_id: candidate.candidate_id.clone(),
                        expected,
                        actual,
                    });
                }
            }
        }
        None
    }
}
impl Default for DriftDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_trace_rs::trace;
    use rts_domain::aggregates::form34a::CandidateResult;
    use uuid::Uuid;

    fn tally(candidate_id: &str, votes: u32) -> Form34C {
        let mut f = Form34C::new(Uuid::new_v4());
        f.candidate_totals = vec![CandidateResult {
            candidate_id: candidate_id.into(),
            candidate_name: String::new(),
            votes,
        }];
        f
    }

    #[trace("FR-M10-022")]
    #[test]
    fn identical_tallies_produce_no_drift() {
        let mut d = DriftDetector::new();
        assert!(d.record_tally("agg-a", &tally("c1", 100)).is_none());
        assert!(d.record_tally("agg-b", &tally("c1", 100)).is_none());
    }

    #[trace("FR-M10-022")]
    #[test]
    fn divergent_tally_raises_drift_report() {
        let mut d = DriftDetector::new();
        d.record_tally("agg-a", &tally("c1", 100));
        let report = d
            .record_tally("agg-b", &tally("c1", 95))
            .expect("drift expected");
        assert_eq!(report.candidate_id, "c1");
        // Reference tally is the first HashMap value; order is not guaranteed,
        // so assert the pair {expected, actual} is {100, 95} without fixing which.
        assert_ne!(report.expected, report.actual);
        assert!(report.expected == 100 || report.expected == 95);
        assert!(report.actual == 100 || report.actual == 95);
    }
}
