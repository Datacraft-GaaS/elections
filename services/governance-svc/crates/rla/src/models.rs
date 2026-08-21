use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditRun {
    pub run_id: String,
    pub election_id: String,
    pub contest_id: String,
    pub risk_limit: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SampledBallot {
    pub ballot_id: String,
    pub batch_id: String,
    pub position: u32,
    pub manifest_entry: String,
    pub reported_vote: Option<String>,
    pub audited_vote: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntropySource {
    pub source_id: String,
    pub beacon_uri: String,
    pub pulse_index: u64,
    pub output_value: String,
    pub fetched_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditReport {
    pub report_id: String,
    pub run_id: String,
    pub risk_achieved: f64,
    pub ballots_audited: u32,
    pub outcome: String,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditManifest {
    pub manifest_id: String,
    pub election_id: String,
    pub total_ballots: u64,
    pub batches: Vec<BatchEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchEntry {
    pub batch_id: String,
    pub num_ballots: u32,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WitnessRecord {
    pub witness_id: String,
    pub run_id: String,
    pub witness_name: String,
    pub affiliation: String,
    pub signature: String,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TStatResult {
    pub run_id: String,
    pub t_statistic: f64,
    pub risk_limit: f64,
    pub passes: bool,
    pub computed_at: String,
}
