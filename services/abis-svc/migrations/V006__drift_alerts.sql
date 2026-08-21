-- Model drift alerts: logged when FMR/FNMR exceeds configured tolerance.
-- Data class D (aggregate, no PII) per ADR-0016.
CREATE TABLE IF NOT EXISTS eel_drift_alerts (
    alert_id         TEXT        PRIMARY KEY,
    drift_type       TEXT        NOT NULL,  -- 'fmr', 'fnmr', 'per_group'
    value            REAL        NOT NULL,
    threshold        REAL        NOT NULL,
    modality         TEXT,
    demographic_group TEXT,
    detected_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at  TIMESTAMPTZ,
    model_card_version TEXT
);

CREATE INDEX IF NOT EXISTS idx_eel_drift_alerts_type       ON eel_drift_alerts(drift_type);
CREATE INDEX IF NOT EXISTS idx_eel_drift_alerts_detected   ON eel_drift_alerts(detected_at DESC);
