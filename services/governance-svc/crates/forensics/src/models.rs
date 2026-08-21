use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnalysisMethod {
    Rd,  // Regression Discontinuity
    Iv,  // Instrumental Variables
    DiD, // Difference-in-Differences
}

impl Default for AnalysisMethod {
    fn default() -> Self {
        Self::Rd
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CycleStatus {
    Draft,
    Running,
    Complete,
    Published,
}

impl Default for CycleStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReferralTarget {
    Frc,
    Knbs,
    Odpp,
}

impl Default for ReferralTarget {
    fn default() -> Self {
        Self::Odpp
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisCycle {
    pub id: String,
    pub election_id: String,
    pub method: AnalysisMethod,
    pub status: CycleStatus,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EstimateRecord {
    pub id: String,
    pub cycle_id: String,
    pub constituency_id: String,
    pub estimate_value: f64,
    pub confidence_interval_low: f64,
    pub confidence_interval_high: f64,
    pub p_value: f64,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReferralPacket {
    pub id: String,
    pub cycle_id: String,
    pub referred_to: ReferralTarget,
    pub referral_basis: String,
    pub content_url: String,
    pub referred_at: String,
    pub acknowledged_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplicationPackage {
    pub id: String,
    pub cycle_id: String,
    pub code_url: String,
    pub data_hash: String,
    pub environment_spec: String,
    pub doi: Option<String>,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublicReport {
    pub id: String,
    pub cycle_id: String,
    pub title: String,
    pub summary: String,
    pub replication_package_id: Option<String>,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateCycleRequest {
    pub election_id: String,
    pub method: AnalysisMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateReferralRequest {
    pub referred_to: ReferralTarget,
    pub referral_basis: String,
    pub content_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublishReplicationRequest {
    pub code_url: String,
    pub data_hash: String,
    pub environment_spec: String,
    pub doi: Option<String>,
}
