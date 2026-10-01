use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnalysisMethod {
    /// Regression Discontinuity.
    #[serde(rename = "RD")]
    Rd,
    /// Instrumental Variables.
    #[serde(rename = "IV")]
    Iv,
    /// Difference-in-Differences.
    #[serde(rename = "DIFF_IN_DIFF")]
    DiD,
}

impl Default for AnalysisMethod {
    fn default() -> Self {
        Self::Rd
    }
}

impl AnalysisMethod {
    /// The contract wire value, e.g. `RD`.
    ///
    /// Derived from the serde representation rather than restated, so the
    /// handler messages and the serialized enum cannot drift apart.
    pub fn wire(&self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default()
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

impl CycleStatus {
    /// The contract wire value, e.g. `DRAFT`.
    ///
    /// Derived from the serde representation, as with `AnalysisMethod::wire`.
    pub fn wire(&self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default()
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
    /// Why the run cannot progress, when it cannot.
    ///
    /// The estimators themselves (RD, IV, DiD) live in
    /// `services/governance-svc/py/forensics-ml`, a separate Python
    /// distribution with no in-process or IPC bridge in this workspace. A run
    /// therefore transitions to `Running` and records the reason here instead of
    /// inventing an estimate. Empty means no run has been blocked.
    pub blocked_reason: Option<String>,
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
