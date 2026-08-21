//! Batch deduplication job management.

use crate::adjudication::{is_ambiguous, open_adjudication_case};
use crate::domain::{AdjudicationCase, TemplateIndex};
use crate::pipeline::MatchPipeline;

pub struct BatchDedupRun {
    pub job_id: String,
    pub election_id: String,
    pub mode: String,
}

pub fn new_batch_job(election_id: &str, mode: &str) -> BatchDedupRun {
    BatchDedupRun {
        job_id: uuid::Uuid::new_v4().to_string(),
        election_id: election_id.to_string(),
        mode: mode.to_string(),
    }
}

/// Runs pairwise 1:1 matching over the index (bounded to first 1000 entries).
/// Returns adjudication cases for ambiguous pairs (score in [0.5, 0.8)).
pub fn run_dedup_batch(
    matcher: &dyn MatchPipeline,
    index: &[TemplateIndex],
    threshold: f32,
) -> Vec<AdjudicationCase> {
    let bounded: &[TemplateIndex] = if index.len() > 1000 {
        &index[..1000]
    } else {
        index
    };
    let mut cases = Vec::new();
    for i in 0..bounded.len() {
        for j in (i + 1)..bounded.len() {
            let result = matcher.match_11(
                &bounded[i].template_hash,
                &bounded[j].template_hash,
                threshold,
            );
            if is_ambiguous(result.confidence_score, 0.5, threshold) {
                let voter_ids = vec![bounded[i].voter_id.clone(), bounded[j].voter_id.clone()];
                cases.push(open_adjudication_case(&result, voter_ids));
            }
        }
    }
    cases
}
