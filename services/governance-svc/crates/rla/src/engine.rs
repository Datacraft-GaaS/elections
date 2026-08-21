use crate::models::{AuditManifest, SampledBallot};

pub struct ShangrlaEngine;

impl ShangrlaEngine {
    pub fn compute_t_statistic(_ballots: &[SampledBallot]) -> f64 {
        0.0
    }

    pub fn should_trigger_recount(t: f64, risk_limit: f64) -> bool {
        t > risk_limit
    }

    pub fn draw_sample(_manifest: &AuditManifest, _seed: &str, _n: usize) -> Vec<String> {
        vec![]
    }
}
