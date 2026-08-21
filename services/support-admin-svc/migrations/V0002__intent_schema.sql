-- support-admin-svc: intent preservation domain tables (WP-5.5, M27).

-- ---- Pre-mark Validation Events (M27) ---------------------------------------

CREATE TABLE IF NOT EXISTS eel_premark_validations (
    id                          UUID PRIMARY KEY,
    station_code                TEXT NOT NULL,
    contest_id                  TEXT NOT NULL,
    -- Image is NOT stored; only inference outcome is recorded.
    inference_outcome           TEXT NOT NULL,
    model_version               TEXT NOT NULL,
    device_id                   TEXT NOT NULL,
    image_delete_elapsed_ms     INTEGER NOT NULL CHECK (image_delete_elapsed_ms <= 1000),
    event_at                    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_pmv_station ON eel_premark_validations (station_code);
CREATE INDEX IF NOT EXISTS idx_pmv_outcome ON eel_premark_validations (inference_outcome);

-- ---- Spoilt Ballot Events (M27, Elections Act s.54) -------------------------

CREATE TABLE IF NOT EXISTS eel_spoilt_ballot_events (
    id                      UUID PRIMARY KEY,
    station_code            TEXT NOT NULL,
    contest_id              TEXT NOT NULL,
    reason_code             TEXT NOT NULL,
    officer_id              TEXT NOT NULL,
    witness_ids             TEXT[] NOT NULL DEFAULT '{}',
    replacement_issued      BOOLEAN NOT NULL DEFAULT FALSE,
    event_at                TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_sbe_station ON eel_spoilt_ballot_events (station_code);

-- ---- Adjudication Events (M27) ----------------------------------------------

CREATE TABLE IF NOT EXISTS eel_adjudication_events (
    id                      UUID PRIMARY KEY,
    ballot_id               TEXT NOT NULL,
    station_code            TEXT NOT NULL,
    contest_id              TEXT NOT NULL,
    adjudicator_id          TEXT NOT NULL,
    reason_code             TEXT NOT NULL CHECK (reason_code IN (
                                'A1','A2','A3','A4','A5','A6','A7','A8',
                                'V1','V2','V3')),
    decision                TEXT NOT NULL CHECK (decision IN ('REJECT','ACCEPT')),
    -- MinIO reference to JPG photo evidence; required for rejection codes A2-A7.
    photo_evidence_ref      TEXT,
    adjudicated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_adj_station ON eel_adjudication_events (station_code);
CREATE INDEX IF NOT EXISTS idx_adj_decision ON eel_adjudication_events (decision);
CREATE INDEX IF NOT EXISTS idx_adj_reason ON eel_adjudication_events (reason_code);

-- ---- Rejection Rate Alerts (M27) -------------------------------------------

CREATE TABLE IF NOT EXISTS eel_rejection_rate_alerts (
    id                  UUID PRIMARY KEY,
    station_code        TEXT NOT NULL,
    contest_id          TEXT NOT NULL,
    rejection_rate      DOUBLE PRECISION NOT NULL,
    national_mean       DOUBLE PRECISION NOT NULL,
    z_score             DOUBLE PRECISION NOT NULL,
    acknowledged        BOOLEAN NOT NULL DEFAULT FALSE,
    acknowledged_by     TEXT,
    alerted_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_rra_station ON eel_rejection_rate_alerts (station_code);
CREATE INDEX IF NOT EXISTS idx_rra_ack ON eel_rejection_rate_alerts (acknowledged);

-- ---- Adjudication disable toggle audit (M27) --------------------------------

CREATE TABLE IF NOT EXISTS eel_adjudication_toggle_audit (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    station_code    TEXT NOT NULL,
    enabled         BOOLEAN NOT NULL,
    toggled_by      TEXT NOT NULL,
    reason          TEXT NOT NULL,
    toggled_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
