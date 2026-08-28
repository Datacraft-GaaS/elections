-- eel AI platform contracts schema (M26 anomaly detection, M44 turnout
-- forecasting, M61 secrecy red teaming, M28 AI model registry).
-- Idempotent per repo convention; applied by the service deploy tooling.

-- ---------------------------------------------------------------------------
-- M26 anomaly detection
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS ai_anomaly_alerts (
    alert_id         UUID           NOT NULL DEFAULT gen_random_uuid(),
    alert_type       TEXT           NOT NULL,
    severity         TEXT           NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    entity_ref       TEXT           NOT NULL,
    detected_at      TIMESTAMPTZ    NOT NULL,
    status           TEXT           NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'investigating', 'dismissed', 'escalated')),
    explanation      TEXT,
    model_id         UUID,
    confidence_score DOUBLE PRECISION CHECK (confidence_score BETWEEN 0 AND 1),
    created_at       TIMESTAMPTZ    NOT NULL DEFAULT now(),
    PRIMARY KEY (alert_id)
);

CREATE INDEX IF NOT EXISTS ai_anomaly_alerts_detected_at_idx
    ON ai_anomaly_alerts (detected_at DESC);
CREATE INDEX IF NOT EXISTS ai_anomaly_alerts_type_severity_idx
    ON ai_anomaly_alerts (alert_type, severity);

CREATE TABLE IF NOT EXISTS ai_anomaly_adjudications (
    adjudication_id UUID        NOT NULL DEFAULT gen_random_uuid(),
    alert_id        UUID        NOT NULL REFERENCES ai_anomaly_alerts(alert_id),
    verdict         TEXT        NOT NULL CHECK (verdict IN ('true_positive', 'false_positive', 'investigating')),
    notes           TEXT,
    escalate_to     TEXT        NOT NULL CHECK (escalate_to IN ('IAB', 'ECC', 'none')),
    decided_by_hash TEXT        NOT NULL,
    decided_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (adjudication_id)
);

CREATE INDEX IF NOT EXISTS ai_anomaly_adjudications_alert_idx
    ON ai_anomaly_adjudications (alert_id);

CREATE TABLE IF NOT EXISTS ai_ingested_events (
    event_id    TEXT        NOT NULL,
    batch_id    TEXT        NOT NULL,
    event_type  TEXT        NOT NULL,
    payload     JSONB       NOT NULL DEFAULT '{}'::jsonb,
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (event_id)
);

-- ---------------------------------------------------------------------------
-- M44 turnout forecasting
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS ai_turnout_forecasts (
    forecast_id            UUID             NOT NULL DEFAULT gen_random_uuid(),
    election_id            UUID             NOT NULL,
    geo_scope              TEXT             NOT NULL,
    predicted_turnout_pct  DOUBLE PRECISION NOT NULL CHECK (predicted_turnout_pct BETWEEN 0 AND 1),
    ci_lower               DOUBLE PRECISION NOT NULL CHECK (ci_lower BETWEEN 0 AND 1),
    ci_upper               DOUBLE PRECISION NOT NULL CHECK (ci_upper BETWEEN 0 AND 1),
    ci_level               DOUBLE PRECISION NOT NULL CHECK (ci_level BETWEEN 0 AND 1),
    model_id               UUID             NOT NULL,
    generated_at           TIMESTAMPTZ      NOT NULL DEFAULT now(),
    horizon_hours          INT              NOT NULL CHECK (horizon_hours BETWEEN 1 AND 168),
    registered_voters      BIGINT,
    PRIMARY KEY (forecast_id),
    UNIQUE (election_id, geo_scope, horizon_hours, generated_at)
);

CREATE INDEX IF NOT EXISTS ai_turnout_forecasts_lookup_idx
    ON ai_turnout_forecasts (election_id, geo_scope, horizon_hours, generated_at DESC);

CREATE TABLE IF NOT EXISTS ai_forecast_accuracy_reports (
    report_id    UUID             NOT NULL DEFAULT gen_random_uuid(),
    election_id  UUID             NOT NULL,
    model_id     UUID             NOT NULL,
    mae          DOUBLE PRECISION NOT NULL,
    rmse         DOUBLE PRECISION NOT NULL,
    bias         DOUBLE PRECISION,
    generated_at TIMESTAMPTZ      NOT NULL DEFAULT now(),
    notes        TEXT,
    PRIMARY KEY (report_id)
);

CREATE INDEX IF NOT EXISTS ai_forecast_accuracy_reports_election_idx
    ON ai_forecast_accuracy_reports (election_id, generated_at DESC);

CREATE TABLE IF NOT EXISTS ai_retrain_jobs (
    job_id                    UUID        NOT NULL DEFAULT gen_random_uuid(),
    model_type                TEXT        NOT NULL CHECK (model_type IN ('turnout', 'anomaly')),
    dataset_ref               TEXT        NOT NULL,
    reason                    TEXT        NOT NULL,
    hyperparameter_overrides  JSONB,
    status                    TEXT        NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'running', 'complete', 'failed')),
    submitted_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (job_id)
);

CREATE INDEX IF NOT EXISTS ai_retrain_jobs_active_idx
    ON ai_retrain_jobs (model_type, status) WHERE status IN ('queued', 'running');

-- ---------------------------------------------------------------------------
-- M61 secrecy red teaming
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS ai_redteam_campaigns (
    campaign_id         UUID        NOT NULL DEFAULT gen_random_uuid(),
    campaign_name       TEXT        NOT NULL,
    target_module       TEXT        NOT NULL,
    attack_type         TEXT        NOT NULL CHECK (attack_type IN ('inference', 'timing', 'side_channel', 'model_inversion')),
    scope_description   TEXT        NOT NULL,
    -- Campaigns are authorized up front and begin execution on creation;
    -- there is no separate start endpoint in the contract.
    status              TEXT        NOT NULL DEFAULT 'running' CHECK (status IN ('planned', 'running', 'complete', 'aborted')),
    authorised_by_hash  TEXT        NOT NULL,
    authorisation_ref   TEXT        NOT NULL,
    overall_risk        TEXT        CHECK (overall_risk IN ('low', 'medium', 'high', 'critical')),
    report_ref          TEXT,
    planned_start_date  DATE,
    planned_end_date    DATE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at        TIMESTAMPTZ,
    PRIMARY KEY (campaign_id)
);

CREATE TABLE IF NOT EXISTS ai_redteam_findings (
    finding_id          UUID              NOT NULL DEFAULT gen_random_uuid(),
    campaign_id         UUID              NOT NULL REFERENCES ai_redteam_campaigns(campaign_id),
    vulnerability_type  TEXT              NOT NULL,
    severity            TEXT              NOT NULL CHECK (severity IN ('info', 'low', 'medium', 'high', 'critical')),
    description         TEXT              NOT NULL,
    reproduction_steps  TEXT              NOT NULL,
    remediation         TEXT              NOT NULL,
    cvss_score          DOUBLE PRECISION  CHECK (cvss_score BETWEEN 0 AND 10),
    evidence_ref        TEXT,
    recorded_at         TIMESTAMPTZ       NOT NULL DEFAULT now(),
    PRIMARY KEY (finding_id)
);

CREATE INDEX IF NOT EXISTS ai_redteam_findings_campaign_idx
    ON ai_redteam_findings (campaign_id, recorded_at DESC);

-- ---------------------------------------------------------------------------
-- M28 AI model registry
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS ai_registry_models (
    model_id             UUID        NOT NULL DEFAULT gen_random_uuid(),
    model_name           TEXT        NOT NULL,
    model_version        TEXT        NOT NULL,
    model_type           TEXT        NOT NULL CHECK (model_type IN ('anomaly', 'speech', 'forecast', 'redteam')),
    artifact_hash        TEXT        NOT NULL CHECK (artifact_hash ~ '^[0-9a-f]{64}$'),
    training_dataset_ref TEXT        NOT NULL,
    fairness_report_ref  TEXT        NOT NULL,
    registered_by_hash   TEXT        NOT NULL CHECK (registered_by_hash ~ '^[0-9a-f]{64}$'),
    status               TEXT        NOT NULL DEFAULT 'staging' CHECK (status IN ('staging', 'active', 'deprecated')),
    notes                TEXT,
    registered_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (model_id)
);

CREATE INDEX IF NOT EXISTS ai_registry_models_type_status_idx
    ON ai_registry_models (model_type, status, registered_at DESC);

CREATE TABLE IF NOT EXISTS ai_registry_deployments (
    deployment_id    UUID        NOT NULL DEFAULT gen_random_uuid(),
    model_id         UUID        NOT NULL REFERENCES ai_registry_models(model_id),
    target_env       TEXT        NOT NULL CHECK (target_env IN ('staging', 'production')),
    status           TEXT        NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'active', 'rolled_back')),
    approved_by_hash TEXT        NOT NULL CHECK (approved_by_hash ~ '^[0-9a-f]{64}$'),
    governance_ref   TEXT        NOT NULL,
    notes            TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (deployment_id)
);

CREATE INDEX IF NOT EXISTS ai_registry_deployments_model_idx
    ON ai_registry_deployments (model_id, created_at DESC);

CREATE TABLE IF NOT EXISTS ai_registry_audit_log (
    entry_id        UUID        NOT NULL DEFAULT gen_random_uuid(),
    model_id        UUID        NOT NULL,
    action          TEXT        NOT NULL CHECK (action IN ('registered', 'deployed', 'deprecated', 'status_changed')),
    actor_hash      TEXT        NOT NULL,
    payload_summary TEXT,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (entry_id)
);

CREATE INDEX IF NOT EXISTS ai_registry_audit_log_model_idx
    ON ai_registry_audit_log (model_id, occurred_at DESC);

-- ---------------------------------------------------------------------------
-- ADR-0011 idempotency: one response snapshot per (key, operation).
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS ai_platform_request_fingerprints (
    idempotency_key     TEXT        NOT NULL,
    operation           TEXT        NOT NULL,
    request_fingerprint TEXT        NOT NULL,
    response_status     INT         NOT NULL,
    response_body       JSONB       NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (idempotency_key, operation)
);
