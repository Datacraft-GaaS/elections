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

//! Storage contract for the AI platform API handlers.
//!
//! Two implementations live here: [`pg::PgStore`] (sqlx runtime queries
//! against the schema in `services/ai-platform-svc/migrations`, used in
//! production) and [`memory::InMemoryStore`] (used by hermetic tests and
//! local tooling).

pub mod memory;
pub mod pg;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Errors surfaced by store implementations. Handlers map these onto
/// RFC 9457 problem responses.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("resource not found")]
    NotFound,
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Unprocessable(String),
    #[error("{0}")]
    Internal(String),
}

// ---------------------------------------------------------------------------
// Anomaly detection (M26)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct AnomalyAlertOut {
    pub alert_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub entity_ref: String,
    pub detected_at: DateTime<Utc>,
    pub status: String,
    pub explanation: Option<String>,
    pub model_id: Option<Uuid>,
    pub confidence_score: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct AlertQuery {
    pub severity: Option<String>,
    pub status: Option<String>,
    pub alert_type: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdjudicationOut {
    pub adjudication_id: Uuid,
    pub alert_id: Uuid,
    pub verdict: String,
    pub notes: Option<String>,
    pub escalate_to: String,
    pub decided_by_hash: String,
    pub decided_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AdjudicationCommand {
    pub alert_id: Uuid,
    pub verdict: String,
    pub notes: Option<String>,
    pub escalate_to: String,
    pub decided_by_hash: String,
    pub decided_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryBucket {
    pub alert_type: String,
    pub severity: String,
    pub count: i64,
    pub open_count: i64,
    pub escalated_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventItemIn {
    pub event_id: String,
    pub event_type: String,
    #[serde(default)]
    pub payload: Value,
}

// ---------------------------------------------------------------------------
// Turnout forecasting (M44)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ConfidenceIntervalOut {
    pub lower: f64,
    pub upper: f64,
    pub level: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TurnoutForecastOut {
    pub forecast_id: Uuid,
    pub election_id: Uuid,
    pub geo_scope: String,
    pub predicted_turnout_pct: f64,
    pub confidence_interval: ConfidenceIntervalOut,
    pub model_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub horizon_hours: i32,
    pub registered_voters: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct ReportQuery {
    pub model_id: Option<Uuid>,
    pub election_id: Option<Uuid>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccuracyReportOut {
    pub report_id: Uuid,
    pub election_id: Uuid,
    pub model_id: Uuid,
    pub mae: f64,
    pub rmse: f64,
    pub bias: Option<f64>,
    pub generated_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewRetrainJob {
    pub model_type: String,
    pub dataset_ref: String,
    pub reason: String,
    pub hyperparameter_overrides: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RetrainJobOut {
    pub job_id: Uuid,
    pub status: String,
    pub model_type: String,
    pub submitted_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Secrecy red teaming (M61)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NewCampaign {
    pub campaign_name: String,
    pub target_module: String,
    pub attack_type: String,
    pub scope_description: String,
    pub authorised_by_hash: String,
    pub authorisation_ref: String,
    pub planned_start_date: Option<chrono::NaiveDate>,
    pub planned_end_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignOut {
    pub campaign_id: Uuid,
    pub campaign_name: String,
    pub target_module: String,
    pub attack_type: String,
    pub scope_description: String,
    pub status: String,
    pub authorised_by_hash: String,
    pub authorisation_ref: String,
    pub overall_risk: Option<String>,
    pub report_ref: Option<String>,
    pub findings_count: i64,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct CampaignQuery {
    pub status: Option<String>,
    pub attack_type: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct NewFinding {
    pub vulnerability_type: String,
    pub severity: String,
    pub description: String,
    pub reproduction_steps: String,
    pub remediation: String,
    pub cvss_score: Option<f64>,
    pub evidence_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FindingOut {
    pub finding_id: Uuid,
    pub campaign_id: Uuid,
    pub vulnerability_type: String,
    pub severity: String,
    pub description: String,
    pub reproduction_steps: String,
    pub remediation: String,
    pub cvss_score: Option<f64>,
    pub evidence_ref: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ClosureCommand {
    pub overall_risk: String,
    pub report_ref: String,
    pub sign_off_hash: String,
    pub closure_notes: Option<String>,
    pub completed_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// AI model registry (M28)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NewModel {
    pub model_name: String,
    pub model_version: String,
    pub model_type: String,
    pub artifact_hash: String,
    pub training_dataset_ref: String,
    pub fairness_report_ref: String,
    pub registered_by_hash: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelOut {
    pub model_id: Uuid,
    pub model_name: String,
    pub model_version: String,
    pub model_type: String,
    pub artifact_hash: String,
    pub training_dataset_ref: String,
    pub fairness_report_ref: String,
    pub registered_by_hash: String,
    pub status: String,
    pub registered_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ModelQuery {
    pub model_type: Option<String>,
    pub status: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct NewDeployment {
    pub target_env: String,
    pub approved_by_hash: String,
    pub governance_ref: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeploymentOut {
    pub deployment_id: Uuid,
    pub model_id: Uuid,
    pub target_env: String,
    pub status: String,
    pub approved_by_hash: String,
    pub governance_ref: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    pub model_id: Option<Uuid>,
    pub action: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditEntryOut {
    pub entry_id: Uuid,
    pub model_id: Uuid,
    pub action: String,
    pub actor_hash: String,
    pub payload_summary: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Idempotency (ADR-0011)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct IdemRecord {
    pub request_fingerprint: String,
    pub response_status: u16,
    pub response_body: Value,
}

/// Paginated result. `total_count` is the number of rows matching the
/// filter regardless of the window.
#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total_count: i64,
}

/// Storage abstraction shared by the Postgres and in-memory
/// implementations. All writes are durable in the Postgres implementation;
/// the in-memory implementation exists for hermetic tests.
#[async_trait]
pub trait AiStore: Send + Sync {
    // ADR-0011 idempotency
    async fn idempotency_lookup(
        &self,
        key: &str,
        operation: &str,
    ) -> Result<Option<IdemRecord>, StoreError>;
    async fn idempotency_save(
        &self,
        key: &str,
        operation: &str,
        request_fingerprint: &str,
        response_status: u16,
        response_body: Value,
    ) -> Result<(), StoreError>;

    // M26 anomaly detection
    async fn list_anomaly_alerts(&self, q: &AlertQuery) -> Result<Page<AnomalyAlertOut>, StoreError>;
    async fn get_anomaly_alert(&self, alert_id: Uuid) -> Result<Option<AnomalyAlertOut>, StoreError>;
    async fn adjudicate_alert(
        &self,
        cmd: &AdjudicationCommand,
    ) -> Result<AdjudicationOut, StoreError>;
    async fn anomaly_summary(
        &self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<SummaryBucket>, StoreError>;
    /// Inserts a batch of events, deduplicating on `event_id`. Returns
    /// `(accepted_count, rejected_event_ids)` where rejected ids are the
    /// duplicates already present in the store.
    async fn ingest_events(
        &self,
        batch_id: &str,
        events: Vec<EventItemIn>,
    ) -> Result<(i64, Vec<String>), StoreError>;

    // M44 turnout forecasting
    async fn get_turnout_forecast(
        &self,
        election_id: Uuid,
        geo_scope: &str,
        horizon_hours: i32,
    ) -> Result<Option<TurnoutForecastOut>, StoreError>;
    async fn list_accuracy_reports(
        &self,
        q: &ReportQuery,
    ) -> Result<Page<AccuracyReportOut>, StoreError>;
    /// Enqueues a retrain job. Fails with [`StoreError::Conflict`] when an
    /// active (queued or running) job already exists for the model type.
    async fn create_retrain_job(&self, job: &NewRetrainJob) -> Result<RetrainJobOut, StoreError>;

    // M61 secrecy red teaming
    async fn create_campaign(&self, c: &NewCampaign) -> Result<CampaignOut, StoreError>;
    async fn list_campaigns(&self, q: &CampaignQuery) -> Result<Page<CampaignOut>, StoreError>;
    async fn get_campaign(&self, campaign_id: Uuid) -> Result<Option<CampaignOut>, StoreError>;
    /// Appends a finding. Fails with [`StoreError::NotFound`] when the
    /// campaign does not exist and [`StoreError::Unprocessable`] when the
    /// campaign is not in the running state.
    async fn add_finding(
        &self,
        campaign_id: Uuid,
        f: &NewFinding,
        recorded_at: DateTime<Utc>,
    ) -> Result<FindingOut, StoreError>;
    async fn complete_campaign(
        &self,
        campaign_id: Uuid,
        c: &ClosureCommand,
    ) -> Result<CampaignOut, StoreError>;

    // M28 AI model registry
    async fn register_model(&self, m: &NewModel) -> Result<ModelOut, StoreError>;
    async fn list_models(&self, q: &ModelQuery) -> Result<Page<ModelOut>, StoreError>;
    async fn get_model(&self, model_id: Uuid) -> Result<Option<ModelOut>, StoreError>;
    async fn deploy_model(
        &self,
        model_id: Uuid,
        d: &NewDeployment,
    ) -> Result<DeploymentOut, StoreError>;
    async fn list_audit_log(&self, q: &AuditQuery) -> Result<Page<AuditEntryOut>, StoreError>;
}
