//! Postgres [`AiStore`] implementation backed by sqlx runtime queries
//! (no compile-time macros) against the schema in
//! `services/ai-platform-svc/migrations/V1__ai_platform_contracts.sql`.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{QueryBuilder, Row};
use uuid::Uuid;

use super::{
    AccuracyReportOut, AdjudicationCommand, AdjudicationOut, AiStore, AlertQuery,
    AnomalyAlertOut, AuditEntryOut, AuditQuery, CampaignOut, CampaignQuery, ClosureCommand,
    DeploymentOut, EventItemIn, FindingOut, IdemRecord, ModelOut, ModelQuery, NewCampaign,
    NewDeployment, NewFinding, NewModel, NewRetrainJob, Page, ReportQuery, RetrainJobOut,
    StoreError, SummaryBucket, TurnoutForecastOut,
};

const CONFICT_ACTIVE_JOB: &str = "an active retrain job already exists for this model type";

/// Postgres-backed store. Cloneable; all clones share the pool.
#[derive(Clone)]
pub struct PgStore {
    pool: PgPool,
}

fn internal<E: std::fmt::Display>(e: E) -> StoreError {
    tracing::error!(error = %e, "ai-platform-api store query failed");
    StoreError::Internal("storage query failed".to_string())
}

impl PgStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Builds a lazily connected store; connections are only attempted when
    /// a handler touches the database.
    pub fn connect_lazy(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(32)
            .connect_lazy(url)?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl AiStore for PgStore {
    async fn idempotency_lookup(
        &self,
        key: &str,
        operation: &str,
    ) -> Result<Option<IdemRecord>, StoreError> {
        let row: Option<(String, i32, serde_json::Value)> = sqlx::query_as(
            "SELECT request_fingerprint, response_status, response_body \
             FROM ai_platform_request_fingerprints WHERE idempotency_key = $1 AND operation = $2",
        )
        .bind(key)
        .bind(operation)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        Ok(row.map(|(fp, status, body)| IdemRecord {
            request_fingerprint: fp,
            response_status: status as u16,
            response_body: body,
        }))
    }

    async fn idempotency_save(
        &self,
        key: &str,
        operation: &str,
        request_fingerprint: &str,
        response_status: u16,
        response_body: Value,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO ai_platform_request_fingerprints \
             (idempotency_key, operation, request_fingerprint, response_status, response_body) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (idempotency_key, operation) DO NOTHING",
        )
        .bind(key)
        .bind(operation)
        .bind(request_fingerprint)
        .bind(i32::from(response_status))
        .bind(response_body)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn list_anomaly_alerts(&self, q: &AlertQuery) -> Result<Page<AnomalyAlertOut>, StoreError> {
        let total: i64 = {
            let mut qb = QueryBuilder::new("SELECT COUNT(*) FROM ai_anomaly_alerts WHERE 1=1");
            push_alert_filters(&mut qb, q);
            qb.build_query_scalar()
                .fetch_one(&self.pool)
                .await
                .map_err(internal)?
        };

        let mut qb = QueryBuilder::new(
            "SELECT alert_id::text, alert_type, severity, entity_ref, detected_at, status, \
             explanation, model_id::text, confidence_score FROM ai_anomaly_alerts WHERE 1=1",
        );
        push_alert_filters(&mut qb, q);
        qb.push(" ORDER BY detected_at DESC, alert_id DESC LIMIT ");
        qb.push(q.limit);
        qb.push(" OFFSET ");
        qb.push(q.offset);

        let rows = qb
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let alert_id = Uuid::parse_str(row.get::<String, _>("alert_id").as_str())
                .map_err(|e| StoreError::Internal(e.to_string()))?;
            items.push(AnomalyAlertOut {
                alert_id,
                alert_type: row.get("alert_type"),
                severity: row.get("severity"),
                entity_ref: row.get("entity_ref"),
                detected_at: row.get("detected_at"),
                status: row.get("status"),
                explanation: row.get("explanation"),
                model_id: row
                    .get::<Option<String>, _>("model_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                confidence_score: row.get("confidence_score"),
            });
        }
        Ok(Page { items, total_count: total })
    }

    async fn get_anomaly_alert(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<AnomalyAlertOut>, StoreError> {
        let row = sqlx::query(
            "SELECT alert_id::text, alert_type, severity, entity_ref, detected_at, status, \
             explanation, model_id::text, confidence_score \
             FROM ai_anomaly_alerts WHERE alert_id = $1",
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        row.map(|row| {
            Ok(AnomalyAlertOut {
                alert_id: Uuid::parse_str(row.get::<String, _>("alert_id").as_str())
                    .map_err(|e| StoreError::Internal(e.to_string()))?,
                alert_type: row.get("alert_type"),
                severity: row.get("severity"),
                entity_ref: row.get("entity_ref"),
                detected_at: row.get("detected_at"),
                status: row.get("status"),
                explanation: row.get("explanation"),
                model_id: row
                    .get::<Option<String>, _>("model_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                confidence_score: row.get("confidence_score"),
            })
        })
        .transpose()
    }

    async fn adjudicate_alert(
        &self,
        cmd: &AdjudicationCommand,
    ) -> Result<AdjudicationOut, StoreError> {
        // Verdict to alert-status mapping per contract:
        // false_positive dismisses; true_positive escalates when an
        // escalation target is given, otherwise stays under investigation;
        // investigating keeps the alert under investigation.
        let new_status = match (cmd.verdict.as_str(), cmd.escalate_to.as_str()) {
            ("false_positive", _) => "dismissed",
            ("true_positive", "IAB" | "ECC") => "escalated",
            _ => "investigating",
        };

        let mut tx = self.pool.begin().await.map_err(internal)?;

        let exists: Option<Uuid> =
            sqlx::query_scalar("SELECT alert_id FROM ai_anomaly_alerts WHERE alert_id = $1 FOR UPDATE")
                .bind(cmd.alert_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(internal)?;
        if exists.is_none() {
            return Err(StoreError::NotFound);
        }

        sqlx::query("UPDATE ai_anomaly_alerts SET status = $2 WHERE alert_id = $1")
            .bind(cmd.alert_id)
            .bind(new_status)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;

        let (adjudication_id, decided_at): (String, DateTime<Utc>) = sqlx::query_as(
            "INSERT INTO ai_anomaly_adjudications \
             (alert_id, verdict, notes, escalate_to, decided_by_hash, decided_at) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING adjudication_id::text, decided_at",
        )
        .bind(cmd.alert_id)
        .bind(&cmd.verdict)
        .bind(&cmd.notes)
        .bind(&cmd.escalate_to)
        .bind(&cmd.decided_by_hash)
        .bind(cmd.decided_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;

        tx.commit().await.map_err(internal)?;

        Ok(AdjudicationOut {
            adjudication_id: Uuid::parse_str(adjudication_id.as_str())
                .map_err(|e| StoreError::Internal(e.to_string()))?,
            alert_id: cmd.alert_id,
            verdict: cmd.verdict.clone(),
            notes: cmd.notes.clone(),
            escalate_to: cmd.escalate_to.clone(),
            decided_by_hash: cmd.decided_by_hash.clone(),
            decided_at,
        })
    }

    async fn anomaly_summary(
        &self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<SummaryBucket>, StoreError> {
        let rows = sqlx::query(
            "SELECT alert_type, severity, COUNT(*) AS count, \
             COUNT(*) FILTER (WHERE status = 'open') AS open_count, \
             COUNT(*) FILTER (WHERE status = 'escalated') AS escalated_count \
             FROM ai_anomaly_alerts \
             WHERE ($1::timestamptz IS NULL OR detected_at >= $1) \
             AND ($2::timestamptz IS NULL OR detected_at <= $2) \
             GROUP BY alert_type, severity ORDER BY alert_type, severity",
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|row| SummaryBucket {
                alert_type: row.get("alert_type"),
                severity: row.get("severity"),
                count: row.get::<i64, _>("count"),
                open_count: row.get::<i64, _>("open_count"),
                escalated_count: row.get::<i64, _>("escalated_count"),
            })
            .collect())
    }

    async fn ingest_events(
        &self,
        batch_id: &str,
        events: Vec<EventItemIn>,
    ) -> Result<(i64, Vec<String>), StoreError> {
        let mut tx = self.pool.begin().await.map_err(internal)?;
        let mut accepted: i64 = 0;
        let mut rejected = Vec::new();
        for event in events {
            let inserted = sqlx::query(
                "INSERT INTO ai_ingested_events (event_id, batch_id, event_type, payload) \
                 VALUES ($1, $2, $3, $4) ON CONFLICT (event_id) DO NOTHING",
            )
            .bind(&event.event_id)
            .bind(batch_id)
            .bind(&event.event_type)
            .bind(event.payload)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
            if inserted.rows_affected() == 1 {
                accepted += 1;
            } else {
                rejected.push(event.event_id);
            }
        }
        tx.commit().await.map_err(internal)?;
        Ok((accepted, rejected))
    }

    async fn get_turnout_forecast(
        &self,
        election_id: Uuid,
        geo_scope: &str,
        horizon_hours: i32,
    ) -> Result<Option<TurnoutForecastOut>, StoreError> {
        let row = sqlx::query(
            "SELECT forecast_id::text, election_id::text, geo_scope, predicted_turnout_pct, \
             ci_lower, ci_upper, ci_level, model_id::text, generated_at, horizon_hours, \
             registered_voters FROM ai_turnout_forecasts \
             WHERE election_id = $1 AND geo_scope = $2 AND horizon_hours = $3 \
             ORDER BY generated_at DESC LIMIT 1",
        )
        .bind(election_id)
        .bind(geo_scope)
        .bind(horizon_hours)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        row.map(|row| parse_forecast_row(&row)).transpose()
    }

    async fn list_accuracy_reports(
        &self,
        q: &ReportQuery,
    ) -> Result<Page<AccuracyReportOut>, StoreError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ai_forecast_accuracy_reports \
             WHERE ($1::uuid IS NULL OR model_id = $1) \
             AND ($2::uuid IS NULL OR election_id = $2)",
        )
        .bind(q.model_id)
        .bind(q.election_id)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        let rows = sqlx::query(
            "SELECT report_id::text, election_id::text, model_id::text, mae, rmse, bias, \
             generated_at, notes FROM ai_forecast_accuracy_reports \
             WHERE ($1::uuid IS NULL OR model_id = $1) \
             AND ($2::uuid IS NULL OR election_id = $2) \
             ORDER BY generated_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(q.model_id)
        .bind(q.election_id)
        .bind(q.limit)
        .bind(q.offset)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;

        let items = rows
            .into_iter()
            .map(|row| {
                Ok(AccuracyReportOut {
                    report_id: uuid_from(&row, "report_id")?,
                    election_id: uuid_from(&row, "election_id")?,
                    model_id: uuid_from(&row, "model_id")?,
                    mae: row.get("mae"),
                    rmse: row.get("rmse"),
                    bias: row.get("bias"),
                    generated_at: row.get("generated_at"),
                    notes: row.get("notes"),
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?;
        Ok(Page { items, total_count: total })
    }

    async fn create_retrain_job(&self, job: &NewRetrainJob) -> Result<RetrainJobOut, StoreError> {
        let active: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM ai_retrain_jobs \
             WHERE model_type = $1 AND status IN ('queued', 'running') LIMIT 1",
        )
        .bind(&job.model_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        if active.is_some() {
            return Err(StoreError::Conflict(CONFICT_ACTIVE_JOB.to_string()));
        }

        let (job_id, status, submitted_at): (String, String, DateTime<Utc>) = sqlx::query_as(
            "INSERT INTO ai_retrain_jobs (model_type, dataset_ref, reason, hyperparameter_overrides) \
             VALUES ($1, $2, $3, $4) RETURNING job_id::text, status, submitted_at",
        )
        .bind(&job.model_type)
        .bind(&job.dataset_ref)
        .bind(&job.reason)
        .bind(job.hyperparameter_overrides.clone())
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        Ok(RetrainJobOut {
            job_id: Uuid::parse_str(job_id.as_str()).map_err(|e| StoreError::Internal(e.to_string()))?,
            status,
            model_type: job.model_type.clone(),
            submitted_at,
        })
    }

    async fn create_campaign(&self, c: &NewCampaign) -> Result<CampaignOut, StoreError> {
        // Campaigns are authorised up front and begin execution on creation;
        // the contract has no separate start transition.
        let (campaign_id, created_at): (String, DateTime<Utc>) = sqlx::query_as(
            "INSERT INTO ai_redteam_campaigns \
             (campaign_name, target_module, attack_type, scope_description, status, \
              authorised_by_hash, authorisation_ref, planned_start_date, planned_end_date) \
             VALUES ($1, $2, $3, $4, 'running', $5, $6, $7, $8) \
             RETURNING campaign_id::text, created_at",
        )
        .bind(&c.campaign_name)
        .bind(&c.target_module)
        .bind(&c.attack_type)
        .bind(&c.scope_description)
        .bind(&c.authorised_by_hash)
        .bind(&c.authorisation_ref)
        .bind(c.planned_start_date)
        .bind(c.planned_end_date)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        Ok(CampaignOut {
            campaign_id: Uuid::parse_str(campaign_id.as_str())
                .map_err(|e| StoreError::Internal(e.to_string()))?,
            campaign_name: c.campaign_name.clone(),
            target_module: c.target_module.clone(),
            attack_type: c.attack_type.clone(),
            scope_description: c.scope_description.clone(),
            status: "running".to_string(),
            authorised_by_hash: c.authorised_by_hash.clone(),
            authorisation_ref: c.authorisation_ref.clone(),
            overall_risk: None,
            report_ref: None,
            findings_count: 0,
            created_at,
            completed_at: None,
        })
    }

    async fn list_campaigns(&self, q: &CampaignQuery) -> Result<Page<CampaignOut>, StoreError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ai_redteam_campaigns \
             WHERE ($1::text IS NULL OR status = $1) \
             AND ($2::text IS NULL OR attack_type = $2)",
        )
        .bind(q.status.as_deref())
        .bind(q.attack_type.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        let rows = sqlx::query(
            "SELECT c.campaign_id::text, c.campaign_name, c.target_module, c.attack_type, \
             c.scope_description, c.status, c.authorised_by_hash, c.authorisation_ref, \
             c.overall_risk, c.report_ref, c.created_at, c.completed_at, \
             (SELECT COUNT(*) FROM ai_redteam_findings f WHERE f.campaign_id = c.campaign_id) \
             AS findings_count FROM ai_redteam_campaigns c \
             WHERE ($1::text IS NULL OR c.status = $1) \
             AND ($2::text IS NULL OR c.attack_type = $2) \
             ORDER BY c.created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(q.status.as_deref())
        .bind(q.attack_type.as_deref())
        .bind(q.limit)
        .bind(q.offset)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;

        let items = rows
            .into_iter()
            .map(parse_campaign_row)
            .collect::<Result<Vec<_>, StoreError>>()?;
        Ok(Page { items, total_count: total })
    }

    async fn get_campaign(&self, campaign_id: Uuid) -> Result<Option<CampaignOut>, StoreError> {
        let row = sqlx::query(
            "SELECT c.campaign_id::text, c.campaign_name, c.target_module, c.attack_type, \
             c.scope_description, c.status, c.authorised_by_hash, c.authorisation_ref, \
             c.overall_risk, c.report_ref, c.created_at, c.completed_at, \
             (SELECT COUNT(*) FROM ai_redteam_findings f WHERE f.campaign_id = c.campaign_id) \
             AS findings_count FROM ai_redteam_campaigns c WHERE c.campaign_id = $1",
        )
        .bind(campaign_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        row.map(|row| parse_campaign_row(row)).transpose()
    }

    async fn add_finding(
        &self,
        campaign_id: Uuid,
        f: &NewFinding,
        recorded_at: DateTime<Utc>,
    ) -> Result<FindingOut, StoreError> {
        let mut tx = self.pool.begin().await.map_err(internal)?;

        let status: Option<String> =
            sqlx::query_scalar("SELECT status FROM ai_redteam_campaigns WHERE campaign_id = $1 FOR UPDATE")
                .bind(campaign_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(internal)?;
        match status.as_deref() {
            None => return Err(StoreError::NotFound),
            Some("running") => {}
            Some(_) => {
                return Err(StoreError::Unprocessable(
                    "campaign is not in running status".to_string(),
                ))
            }
        }

        let (finding_id, recorded_at): (String, DateTime<Utc>) = sqlx::query_as(
            "INSERT INTO ai_redteam_findings \
             (campaign_id, vulnerability_type, severity, description, reproduction_steps, \
              remediation, cvss_score, evidence_ref, recorded_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             RETURNING finding_id::text, recorded_at",
        )
        .bind(campaign_id)
        .bind(&f.vulnerability_type)
        .bind(&f.severity)
        .bind(&f.description)
        .bind(&f.reproduction_steps)
        .bind(&f.remediation)
        .bind(f.cvss_score)
        .bind(f.evidence_ref.as_deref())
        .bind(recorded_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;

        tx.commit().await.map_err(internal)?;

        Ok(FindingOut {
            finding_id: Uuid::parse_str(finding_id.as_str())
                .map_err(|e| StoreError::Internal(e.to_string()))?,
            campaign_id,
            vulnerability_type: f.vulnerability_type.clone(),
            severity: f.severity.clone(),
            description: f.description.clone(),
            reproduction_steps: f.reproduction_steps.clone(),
            remediation: f.remediation.clone(),
            cvss_score: f.cvss_score,
            evidence_ref: f.evidence_ref.clone(),
            recorded_at,
        })
    }

    async fn complete_campaign(
        &self,
        campaign_id: Uuid,
        c: &ClosureCommand,
    ) -> Result<CampaignOut, StoreError> {
        let mut tx = self.pool.begin().await.map_err(internal)?;

        let status: Option<String> =
            sqlx::query_scalar("SELECT status FROM ai_redteam_campaigns WHERE campaign_id = $1 FOR UPDATE")
                .bind(campaign_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(internal)?;
        match status.as_deref() {
            None => return Err(StoreError::NotFound),
            Some("running") => {}
            Some(_) => {
                return Err(StoreError::Unprocessable(
                    "campaign is not in running status".to_string(),
                ))
            }
        }

        let updated = sqlx::query(
            "UPDATE ai_redteam_campaigns SET status = 'complete', overall_risk = $2, \
             report_ref = $3, completed_at = $4 WHERE campaign_id = $1",
        )
        .bind(campaign_id)
        .bind(&c.overall_risk)
        .bind(&c.report_ref)
        .bind(c.completed_at)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
        if updated.rows_affected() != 1 {
            return Err(StoreError::Internal(
                "campaign completion update affected no rows".to_string(),
            ));
        }
        tx.commit().await.map_err(internal)?;

        self.get_campaign(campaign_id)
            .await?
            .ok_or_else(|| StoreError::Internal("campaign vanished after update".to_string()))
    }

    async fn register_model(&self, m: &NewModel) -> Result<ModelOut, StoreError> {
        let mut tx = self.pool.begin().await.map_err(internal)?;

        let (model_id, registered_at): (String, DateTime<Utc>) = sqlx::query_as(
            "INSERT INTO ai_registry_models \
             (model_name, model_version, model_type, artifact_hash, training_dataset_ref, \
              fairness_report_ref, registered_by_hash, notes) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             RETURNING model_id::text, registered_at",
        )
        .bind(&m.model_name)
        .bind(&m.model_version)
        .bind(&m.model_type)
        .bind(&m.artifact_hash)
        .bind(&m.training_dataset_ref)
        .bind(&m.fairness_report_ref)
        .bind(&m.registered_by_hash)
        .bind(m.notes.as_deref())
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;

        let model_uuid =
            Uuid::parse_str(model_id.as_str()).map_err(|e| StoreError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO ai_registry_audit_log (model_id, action, actor_hash, payload_summary) \
             VALUES ($1, 'registered', $2, $3)",
        )
        .bind(model_uuid)
        .bind(&m.registered_by_hash)
        .bind(format!("{} v{} ({})", m.model_name, m.model_version, m.model_type))
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

        tx.commit().await.map_err(internal)?;

        Ok(ModelOut {
            model_id: model_uuid,
            model_name: m.model_name.clone(),
            model_version: m.model_version.clone(),
            model_type: m.model_type.clone(),
            artifact_hash: m.artifact_hash.clone(),
            training_dataset_ref: m.training_dataset_ref.clone(),
            fairness_report_ref: m.fairness_report_ref.clone(),
            registered_by_hash: m.registered_by_hash.clone(),
            status: "staging".to_string(),
            registered_at,
            notes: m.notes.clone(),
        })
    }

    async fn list_models(&self, q: &ModelQuery) -> Result<Page<ModelOut>, StoreError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ai_registry_models \
             WHERE ($1::text IS NULL OR model_type = $1) \
             AND ($2::text IS NULL OR status = $2)",
        )
        .bind(q.model_type.as_deref())
        .bind(q.status.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        let rows = sqlx::query(
            "SELECT model_id::text, model_name, model_version, model_type, artifact_hash, \
             training_dataset_ref, fairness_report_ref, registered_by_hash, status, \
             registered_at, notes FROM ai_registry_models \
             WHERE ($1::text IS NULL OR model_type = $1) \
             AND ($2::text IS NULL OR status = $2) \
             ORDER BY registered_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(q.model_type.as_deref())
        .bind(q.status.as_deref())
        .bind(q.limit)
        .bind(q.offset)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;

        let items = rows
            .into_iter()
            .map(|row| {
                Ok(ModelOut {
                    model_id: uuid_from(&row, "model_id")?,
                    model_name: row.get("model_name"),
                    model_version: row.get("model_version"),
                    model_type: row.get("model_type"),
                    artifact_hash: row.get("artifact_hash"),
                    training_dataset_ref: row.get("training_dataset_ref"),
                    fairness_report_ref: row.get("fairness_report_ref"),
                    registered_by_hash: row.get("registered_by_hash"),
                    status: row.get("status"),
                    registered_at: row.get("registered_at"),
                    notes: row.get("notes"),
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?;
        Ok(Page { items, total_count: total })
    }

    async fn get_model(&self, model_id: Uuid) -> Result<Option<ModelOut>, StoreError> {
        let row = sqlx::query(
            "SELECT model_id::text, model_name, model_version, model_type, artifact_hash, \
             training_dataset_ref, fairness_report_ref, registered_by_hash, status, \
             registered_at, notes FROM ai_registry_models WHERE model_id = $1",
        )
        .bind(model_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        row.map(|row| {
            Ok(ModelOut {
                model_id: uuid_from(&row, "model_id")?,
                model_name: row.get("model_name"),
                model_version: row.get("model_version"),
                model_type: row.get("model_type"),
                artifact_hash: row.get("artifact_hash"),
                training_dataset_ref: row.get("training_dataset_ref"),
                fairness_report_ref: row.get("fairness_report_ref"),
                registered_by_hash: row.get("registered_by_hash"),
                status: row.get("status"),
                registered_at: row.get("registered_at"),
                notes: row.get("notes"),
            })
        })
        .transpose()
    }

    async fn deploy_model(
        &self,
        model_id: Uuid,
        d: &NewDeployment,
    ) -> Result<DeploymentOut, StoreError> {
        let mut tx = self.pool.begin().await.map_err(internal)?;

        let version: Option<String> =
            sqlx::query_scalar("SELECT model_version FROM ai_registry_models WHERE model_id = $1 FOR UPDATE")
                .bind(model_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(internal)?;
        let Some(version) = version else {
            return Err(StoreError::NotFound);
        };

        let (deployment_id, created_at): (String, DateTime<Utc>) = sqlx::query_as(
            "INSERT INTO ai_registry_deployments \
             (model_id, target_env, approved_by_hash, governance_ref, notes) \
             VALUES ($1, $2, $3, $4, $5) RETURNING deployment_id::text, created_at",
        )
        .bind(model_id)
        .bind(&d.target_env)
        .bind(&d.approved_by_hash)
        .bind(&d.governance_ref)
        .bind(d.notes.as_deref())
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;

        sqlx::query(
            "INSERT INTO ai_registry_audit_log (model_id, action, actor_hash, payload_summary) \
             VALUES ($1, 'deployed', $2, $3)",
        )
        .bind(model_id)
        .bind(&d.approved_by_hash)
        .bind(format!("deployed {} to {}", version, d.target_env))
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

        tx.commit().await.map_err(internal)?;

        Ok(DeploymentOut {
            deployment_id: Uuid::parse_str(deployment_id.as_str())
                .map_err(|e| StoreError::Internal(e.to_string()))?,
            model_id,
            target_env: d.target_env.clone(),
            status: "pending".to_string(),
            approved_by_hash: d.approved_by_hash.clone(),
            governance_ref: d.governance_ref.clone(),
            created_at,
        })
    }

    async fn list_audit_log(&self, q: &AuditQuery) -> Result<Page<AuditEntryOut>, StoreError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ai_registry_audit_log \
             WHERE ($1::uuid IS NULL OR model_id = $1) \
             AND ($2::text IS NULL OR action = $2)",
        )
        .bind(q.model_id)
        .bind(q.action.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        let rows = sqlx::query(
            "SELECT entry_id::text, model_id::text, action, actor_hash, payload_summary, \
             occurred_at FROM ai_registry_audit_log \
             WHERE ($1::uuid IS NULL OR model_id = $1) \
             AND ($2::text IS NULL OR action = $2) \
             ORDER BY occurred_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(q.model_id)
        .bind(q.action.as_deref())
        .bind(q.limit)
        .bind(q.offset)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;

        let items = rows
            .into_iter()
            .map(|row| {
                Ok(AuditEntryOut {
                    entry_id: uuid_from(&row, "entry_id")?,
                    model_id: uuid_from(&row, "model_id")?,
                    action: row.get("action"),
                    actor_hash: row.get("actor_hash"),
                    payload_summary: row.get("payload_summary"),
                    occurred_at: row.get("occurred_at"),
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?;
        Ok(Page { items, total_count: total })
    }
}

fn push_alert_filters<'a>(qb: &mut sqlx::QueryBuilder<'a, sqlx::Postgres>, q: &'a AlertQuery) {
    if let Some(severity) = &q.severity {
        qb.push(" AND severity = ").push_bind(severity.clone());
    }
    if let Some(status) = &q.status {
        qb.push(" AND status = ").push_bind(status.clone());
    }
    if let Some(alert_type) = &q.alert_type {
        qb.push(" AND alert_type = ").push_bind(alert_type.clone());
    }
    if let Some(from) = q.from {
        qb.push(" AND detected_at >= ").push_bind(from);
    }
    if let Some(to) = q.to {
        qb.push(" AND detected_at <= ").push_bind(to);
    }
}

fn uuid_from(row: &sqlx::postgres::PgRow, column: &str) -> Result<Uuid, StoreError> {
    let s: String = row
        .try_get(column)
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    Uuid::parse_str(&s).map_err(|e| StoreError::Internal(e.to_string()))
}

fn parse_forecast_row(row: &sqlx::postgres::PgRow) -> Result<TurnoutForecastOut, StoreError> {
    Ok(TurnoutForecastOut {
        forecast_id: uuid_from(row, "forecast_id")?,
        election_id: uuid_from(row, "election_id")?,
        geo_scope: row.get("geo_scope"),
        predicted_turnout_pct: row.get("predicted_turnout_pct"),
        confidence_interval: super::ConfidenceIntervalOut {
            lower: row.get("ci_lower"),
            upper: row.get("ci_upper"),
            level: row.get("ci_level"),
        },
        model_id: uuid_from(row, "model_id")?,
        generated_at: row.get("generated_at"),
        horizon_hours: row.get("horizon_hours"),
        registered_voters: row.get("registered_voters"),
    })
}

fn parse_campaign_row(row: sqlx::postgres::PgRow) -> Result<CampaignOut, StoreError> {
    Ok(CampaignOut {
        campaign_id: uuid_from(&row, "campaign_id")?,
        campaign_name: row.get("campaign_name"),
        target_module: row.get("target_module"),
        attack_type: row.get("attack_type"),
        scope_description: row.get("scope_description"),
        status: row.get("status"),
        authorised_by_hash: row.get("authorised_by_hash"),
        authorisation_ref: row.get("authorisation_ref"),
        overall_risk: row.get("overall_risk"),
        report_ref: row.get("report_ref"),
        findings_count: row.get("findings_count"),
        created_at: row.get("created_at"),
        completed_at: row.get("completed_at"),
    })
}
