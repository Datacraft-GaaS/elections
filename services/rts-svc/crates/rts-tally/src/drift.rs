use rts_domain::aggregates::form34c::Form34C;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport { pub aggregator_id: String, pub candidate_id: String, pub expected: u64, pub actual: u64 }

pub struct DriftDetector { tallies: HashMap<String, Form34C> }
impl DriftDetector {
    pub fn new() -> Self { Self { tallies: HashMap::new() } }
    pub fn record_tally(&mut self, aggregator_id: &str, form34c: &Form34C) -> Option<DriftReport> {
        self.tallies.insert(aggregator_id.to_string(), form34c.clone());
        if self.tallies.len() < 2 { return None; }
        let reference = self.tallies.values().next()?.clone();
        for (agg_id, tally) in &self.tallies {
            for candidate in &tally.candidate_totals {
                let expected = reference.candidate_totals.iter().find(|c| c.candidate_id == candidate.candidate_id).map(|c| c.votes as u64).unwrap_or(0);
                let actual = candidate.votes as u64;
                if expected != actual {
                    return Some(DriftReport { aggregator_id: agg_id.clone(), candidate_id: candidate.candidate_id.clone(), expected, actual });
                }
            }
        }
        None
    }
}
impl Default for DriftDetector { fn default() -> Self { Self::new() } }
