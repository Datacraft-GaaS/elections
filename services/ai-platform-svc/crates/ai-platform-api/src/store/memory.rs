//! In-memory [`AiStore`] implementation used by hermetic tests and local
//! tooling. Data is process-local and lost on restart; production traffic
//! uses [`super::pg::PgStore`].

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{
    AccuracyReportOut, AdjudicationCommand, AdjudicationOut, AiStore, AlertQuery,
    AnomalyAlertOut, AuditEntryOut, AuditQuery, CampaignOut, CampaignQuery, ClosureCommand,
    DeploymentOut, EventItemIn, FindingOut, IdemRecord, ModelOut, ModelQuery, NewCampaign,
    NewDeployment, NewFinding, NewModel, NewRetrainJob, Page, ReportQuery, RetrainJobOut,
    StoreError, SummaryBucket, TurnoutForecastOut,
};

#[derive(Default)]
struct MemData {
    alerts: Vec<AnomalyAlertOut>,
    forecasts: Vec<TurnoutForecastOut>,
    reports: Vec<AccuracyReportOut>,
    retrain_jobs: Vec<RetrainJobOut>,
    campaigns: Vec<CampaignOut>,
    findings: Vec<FindingOut>,
    models: Vec<ModelOut>,
    deployments: Vec<DeploymentOut>,
    audit: Vec<AuditEntryOut>,
    event_ids: HashSet<String>,
    idem: HashMap<(String, String), IdemRecord>,
}

/// In-memory store. Cloneable; all clones share the same data.
#[derive(Clone, Default)]
pub struct InMemoryStore {
    data: Arc<RwLock<MemData>>,
}

fn findings_count(data: &MemData, campaign_id: Uuid) -> i64 {
    data.findings
        .iter()
        .filter(|f| f.campaign_id == campaign_id)
        .count() as i64
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "critical" => 3,
        "high" => 2,
        "medium" => 1,
        _ => 0,
    }
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seeding helper for tests and local tooling: inserts an anomaly alert
    /// as the detection pipeline would.
    pub async fn seed_alert(&self, alert: AnomalyAlertOut) {
        self.data.write().await.alerts.push(alert);
    }

    /// Seeding helper for tests and local tooling: inserts a turnout
    /// forecast row as the forecasting pipeline would.
    pub async fn seed_forecast(&self, forecast: TurnoutForecastOut) {
        self.data.write().await.forecasts.push(forecast);
    }

    /// Seeding helper for tests and local tooling: inserts a forecast
    /// accuracy report row as the evaluation pipeline would.
    pub async fn seed_accuracy_report(&self, report: AccuracyReportOut) {
        self.data.write().await.reports.push(report);
    }
}

#[async_trait]
impl AiStore for InMemoryStore {
    async fn idempotency_lookup(
        &self,
        key: &str,
        operation: &str,
    ) -> Result<Option<IdemRecord>, StoreError> {
        let data = self.data.read().await;
        Ok(data.idem.get(&(key.to_string(), operation.to_string())).cloned())
    }

    async fn idempotency_save(
        &self,
        key: &str,
        operation: &str,
        request_fingerprint: &str,
        response_status: u16,
        response_body: Value,
    ) -> Result<(), StoreError> {
        self.data.write().await.idem.insert(
            (key.to_string(), operation.to_string()),
            IdemRecord {
                request_fingerprint: request_fingerprint.to_string(),
                response_status,
                response_body,
            },
        );
        Ok(())
    }

    async fn list_anomaly_alerts(&self, q: &AlertQuery) -> Result<Page<AnomalyAlertOut>, StoreError> {
        let data = self.data.read().await;
        let mut items: Vec<AnomalyAlertOut> = data
            .alerts
            .iter()
            .filter(|a| {
                q.severity.as_ref().map_or(true, |v| a.severity == *v)
                    && q.status.as_ref().map_or(true, |v| a.status == *v)
                    && q.alert_type.as_ref().map_or(true, |v| a.alert_type == *v)
                    && q.from.map_or(true, |t| a.detected_at >= t)
                    && q.to.map_or(true, |t| a.detected_at <= t)
            })
            .cloned()
            .collect();
        let total = items.len() as i64;
        // Most recent first; ties broken by alert_id for stable pagination.
        items.sort_by(|a, b| {
            b.detected_at
                .cmp(&a.detected_at)
                .then(b.alert_id.cmp(&a.alert_id))
        });
        let page: Vec<AnomalyAlertOut> = items
            .into_iter()
            .skip(q.offset.max(0) as usize)
            .take(q.limit.max(0) as usize)
            .collect();
        Ok(Page { items: page, total_count: total })
    }

    async fn get_anomaly_alert(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<AnomalyAlertOut>, StoreError> {
        let data = self.data.read().await;
        Ok(data.alerts.iter().find(|a| a.alert_id == alert_id).cloned())
    }

    async fn adjudicate_alert(
        &self,
        cmd: &AdjudicationCommand,
    ) -> Result<AdjudicationOut, StoreError> {
        let mut data = self.data.write().await;
        let alert = data
            .alerts
            .iter_mut()
            .find(|a| a.alert_id == cmd.alert_id)
            .ok_or(StoreError::NotFound)?;

        // Verdict to alert-status mapping per contract:
        // false_positive dismisses; true_positive escalates when an
        // escalation target is given, otherwise stays under investigation;
        // investigating keeps the alert under investigation.
        alert.status = match (cmd.verdict.as_str(), cmd.escalate_to.as_str()) {
            ("false_positive", _) => "dismissed".to_string(),
            ("true_positive", "IAB" | "ECC") => "escalated".to_string(),
            _ => "investigating".to_string(),
        };

        Ok(AdjudicationOut {
            adjudication_id: Uuid::now_v7(),
            alert_id: cmd.alert_id,
            verdict: cmd.verdict.clone(),
            notes: cmd.notes.clone(),
            escalate_to: cmd.escalate_to.clone(),
            decided_by_hash: cmd.decided_by_hash.clone(),
            decided_at: cmd.decided_at,
        })
    }

    async fn anomaly_summary(
        &self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<SummaryBucket>, StoreError> {
        let data = self.data.read().await;
        let mut buckets: Vec<SummaryBucket> = Vec::new();
        for alert in &data.alerts {
            if from.is_some_and(|t| alert.detected_at < t)
                || to.is_some_and(|t| alert.detected_at > t)
            {
                continue;
            }
            match buckets
                .iter_mut()
                .find(|b| b.alert_type == alert.alert_type && b.severity == alert.severity)
            {
                Some(b) => {
                    b.count += 1;
                    if alert.status == "open" {
                        b.open_count += 1;
                    }
                    if alert.status == "escalated" {
                        b.escalated_count += 1;
                    }
                }
                None => buckets.push(SummaryBucket {
                    alert_type: alert.alert_type.clone(),
                    severity: alert.severity.clone(),
                    count: 1,
                    open_count: i64::from(alert.status == "open"),
                    escalated_count: i64::from(alert.status == "escalated"),
                }),
            }
        }
        buckets.sort_by(|a, b| {
            a.alert_type
                .cmp(&b.alert_type)
                .then(severity_rank(&b.severity).cmp(&severity_rank(&a.severity)))
        });
        Ok(buckets)
    }

    async fn ingest_events(
        &self,
        batch_id: &str,
        events: Vec<EventItemIn>,
    ) -> Result<(i64, Vec<String>), StoreError> {
        let mut data = self.data.write().await;
        let mut rejected = Vec::new();
        let mut accepted: i64 = 0;
        for event in events {
            if data.event_ids.insert(event.event_id.clone()) {
                accepted += 1;
            } else {
                rejected.push(event.event_id);
            }
        }
        let _ = batch_id; // correlation only; events are keyed by event_id
        Ok((accepted, rejected))
    }

    async fn get_turnout_forecast(
        &self,
        election_id: Uuid,
        geo_scope: &str,
        horizon_hours: i32,
    ) -> Result<Option<TurnoutForecastOut>, StoreError> {
        let data = self.data.read().await;
        Ok(data
            .forecasts
            .iter()
            .filter(|f| {
                f.election_id == election_id
                    && f.geo_scope == geo_scope
                    && f.horizon_hours == horizon_hours
            })
            .max_by_key(|f| f.generated_at)
            .cloned())
    }

    async fn list_accuracy_reports(
        &self,
        q: &ReportQuery,
    ) -> Result<Page<AccuracyReportOut>, StoreError> {
        let data = self.data.read().await;
        let mut items: Vec<AccuracyReportOut> = data
            .reports
            .iter()
            .filter(|r| {
                q.model_id.map_or(true, |m| r.model_id == m)
                    && q.election_id.map_or(true, |e| r.election_id == e)
            })
            .cloned()
            .collect();
        let total = items.len() as i64;
        items.sort_by(|a, b| b.generated_at.cmp(&a.generated_at));
        let page = items
            .into_iter()
            .skip(q.offset.max(0) as usize)
            .take(q.limit.max(0) as usize)
            .collect();
        Ok(Page { items: page, total_count: total })
    }

    async fn create_retrain_job(&self, job: &NewRetrainJob) -> Result<RetrainJobOut, StoreError> {
        let mut data = self.data.write().await;
        if data.retrain_jobs.iter().any(|j| {
            j.model_type == job.model_type && (j.status == "queued" || j.status == "running")
        }) {
            return Err(StoreError::Conflict(format!(
                "an active {} retrain job already exists",
                job.model_type
            )));
        }
        let out = RetrainJobOut {
            job_id: Uuid::now_v7(),
            status: "queued".to_string(),
            model_type: job.model_type.clone(),
            submitted_at: Utc::now(),
        };
        data.retrain_jobs.push(out.clone());
        Ok(out)
    }

    async fn create_campaign(&self, c: &NewCampaign) -> Result<CampaignOut, StoreError> {
        let mut data = self.data.write().await;
        let out = CampaignOut {
            campaign_id: Uuid::now_v7(),
            campaign_name: c.campaign_name.clone(),
            target_module: c.target_module.clone(),
            attack_type: c.attack_type.clone(),
            scope_description: c.scope_description.clone(),
            // Campaigns are authorised up front and begin execution on
            // creation; the contract has no separate start transition.
            status: "running".to_string(),
            authorised_by_hash: c.authorised_by_hash.clone(),
            authorisation_ref: c.authorisation_ref.clone(),
            overall_risk: None,
            report_ref: None,
            findings_count: 0,
            created_at: Utc::now(),
            completed_at: None,
        };
        data.campaigns.push(out.clone());
        Ok(out)
    }

    async fn list_campaigns(&self, q: &CampaignQuery) -> Result<Page<CampaignOut>, StoreError> {
        let data = self.data.read().await;
        let mut items: Vec<CampaignOut> = data
            .campaigns
            .iter()
            .filter(|c| {
                q.status.as_ref().map_or(true, |v| c.status == *v)
                    && q.attack_type.as_ref().map_or(true, |v| c.attack_type == *v)
            })
            .map(|c| {
                let mut out = c.clone();
                out.findings_count = findings_count(&data, c.campaign_id);
                out
            })
            .collect();
        let total = items.len() as i64;
        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let page = items
            .into_iter()
            .skip(q.offset.max(0) as usize)
            .take(q.limit.max(0) as usize)
            .collect();
        Ok(Page { items: page, total_count: total })
    }

    async fn get_campaign(&self, campaign_id: Uuid) -> Result<Option<CampaignOut>, StoreError> {
        let data = self.data.read().await;
        Ok(data
            .campaigns
            .iter()
            .find(|c| c.campaign_id == campaign_id)
            .map(|c| {
                let mut out = c.clone();
                out.findings_count = findings_count(&data, campaign_id);
                out
            }))
    }

    async fn add_finding(
        &self,
        campaign_id: Uuid,
        f: &NewFinding,
        recorded_at: DateTime<Utc>,
    ) -> Result<FindingOut, StoreError> {
        let mut data = self.data.write().await;
        let campaign = data
            .campaigns
            .iter()
            .find(|c| c.campaign_id == campaign_id)
            .ok_or(StoreError::NotFound)?;
        if campaign.status != "running" {
            return Err(StoreError::Unprocessable(
                "campaign is not in running status".to_string(),
            ));
        }
        let out = FindingOut {
            finding_id: Uuid::now_v7(),
            campaign_id,
            vulnerability_type: f.vulnerability_type.clone(),
            severity: f.severity.clone(),
            description: f.description.clone(),
            reproduction_steps: f.reproduction_steps.clone(),
            remediation: f.remediation.clone(),
            cvss_score: f.cvss_score,
            evidence_ref: f.evidence_ref.clone(),
            recorded_at,
        };
        data.findings.push(out.clone());
        Ok(out)
    }

    async fn complete_campaign(
        &self,
        campaign_id: Uuid,
        c: &ClosureCommand,
    ) -> Result<CampaignOut, StoreError> {
        let mut data = self.data.write().await;
        let campaign = data
            .campaigns
            .iter_mut()
            .find(|c| c.campaign_id == campaign_id)
            .ok_or(StoreError::NotFound)?;
        if campaign.status != "running" {
            return Err(StoreError::Unprocessable(
                "campaign is not in running status".to_string(),
            ));
        }
        campaign.status = "complete".to_string();
        campaign.overall_risk = Some(c.overall_risk.clone());
        campaign.report_ref = Some(c.report_ref.clone());
        campaign.completed_at = Some(c.completed_at);
        let mut out = campaign.clone();
        out.findings_count = findings_count(&data, campaign_id);
        Ok(out)
    }

    async fn register_model(&self, m: &NewModel) -> Result<ModelOut, StoreError> {
        let mut data = self.data.write().await;
        let model = ModelOut {
            model_id: Uuid::now_v7(),
            model_name: m.model_name.clone(),
            model_version: m.model_version.clone(),
            model_type: m.model_type.clone(),
            artifact_hash: m.artifact_hash.clone(),
            training_dataset_ref: m.training_dataset_ref.clone(),
            fairness_report_ref: m.fairness_report_ref.clone(),
            registered_by_hash: m.registered_by_hash.clone(),
            status: "staging".to_string(),
            registered_at: Utc::now(),
            notes: m.notes.clone(),
        };
        data.audit.push(AuditEntryOut {
            entry_id: Uuid::now_v7(),
            model_id: model.model_id,
            action: "registered".to_string(),
            actor_hash: m.registered_by_hash.clone(),
            payload_summary: Some(format!(
                "{} v{} ({})",
                model.model_name, model.model_version, model.model_type
            )),
            occurred_at: model.registered_at,
        });
        data.models.push(model.clone());
        Ok(model)
    }

    async fn list_models(&self, q: &ModelQuery) -> Result<Page<ModelOut>, StoreError> {
        let data = self.data.read().await;
        let mut items: Vec<ModelOut> = data
            .models
            .iter()
            .filter(|m| {
                q.model_type.as_ref().map_or(true, |v| m.model_type == *v)
                    && q.status.as_ref().map_or(true, |v| m.status == *v)
            })
            .cloned()
            .collect();
        let total = items.len() as i64;
        items.sort_by(|a, b| b.registered_at.cmp(&a.registered_at));
        let page = items
            .into_iter()
            .skip(q.offset.max(0) as usize)
            .take(q.limit.max(0) as usize)
            .collect();
        Ok(Page { items: page, total_count: total })
    }

    async fn get_model(&self, model_id: Uuid) -> Result<Option<ModelOut>, StoreError> {
        let data = self.data.read().await;
        Ok(data.models.iter().find(|m| m.model_id == model_id).cloned())
    }

    async fn deploy_model(
        &self,
        model_id: Uuid,
        d: &NewDeployment,
    ) -> Result<DeploymentOut, StoreError> {
        let mut data = self.data.write().await;
        let model_version = data
            .models
            .iter()
            .find(|m| m.model_id == model_id)
            .ok_or(StoreError::NotFound)?
            .model_version
            .clone();
        let created_at = Utc::now();
        let out = DeploymentOut {
            deployment_id: Uuid::now_v7(),
            model_id,
            target_env: d.target_env.clone(),
            status: "pending".to_string(),
            approved_by_hash: d.approved_by_hash.clone(),
            governance_ref: d.governance_ref.clone(),
            created_at,
        };
        data.audit.push(AuditEntryOut {
            entry_id: Uuid::now_v7(),
            model_id,
            action: "deployed".to_string(),
            actor_hash: d.approved_by_hash.clone(),
            payload_summary: Some(format!(
                "deployed {model_version} to {}",
                d.target_env
            )),
            occurred_at: created_at,
        });
        data.deployments.push(out.clone());
        Ok(out)
    }

    async fn list_audit_log(&self, q: &AuditQuery) -> Result<Page<AuditEntryOut>, StoreError> {
        let data = self.data.read().await;
        let mut items: Vec<AuditEntryOut> = data
            .audit
            .iter()
            .filter(|e| {
                q.model_id.map_or(true, |m| e.model_id == m)
                    && q.action.as_ref().map_or(true, |a| e.action == *a)
            })
            .cloned()
            .collect();
        let total = items.len() as i64;
        items.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
        let page = items
            .into_iter()
            .skip(q.offset.max(0) as usize)
            .take(q.limit.max(0) as usize)
            .collect();
        Ok(Page { items: page, total_count: total })
    }
}
