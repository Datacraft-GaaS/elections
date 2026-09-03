-- RLA API surface — postgres for public contract (no in-memory)
-- Replaces Mutex<HashMap> audits/samples/counts in rla/spec_state.rs

CREATE TABLE IF NOT EXISTS rla_api_audits (
    audit_id        TEXT PRIMARY KEY,
    election_id     TEXT NOT NULL,
    contest_code    TEXT NOT NULL,
    risk_limit      DOUBLE PRECISION NOT NULL CHECK (risk_limit > 0 AND risk_limit <= 1),
    method          TEXT NOT NULL,
    seed            TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'initiated',
    current_round   INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS rla_api_samples (
    audit_id        TEXT NOT NULL REFERENCES rla_api_audits(audit_id) ON DELETE CASCADE,
    round           INTEGER NOT NULL,
    ballot_id_hash  TEXT NOT NULL,
    station_code    TEXT NOT NULL,
    form_id         TEXT NOT NULL,
    PRIMARY KEY (audit_id, ballot_id_hash)
);

CREATE TABLE IF NOT EXISTS rla_api_counts (
    audit_id            TEXT NOT NULL REFERENCES rla_api_audits(audit_id) ON DELETE CASCADE,
    ballot_id_hash      TEXT NOT NULL,
    cvr_candidate_id    TEXT NOT NULL,
    audit_candidate_id  TEXT NOT NULL,
    matches             BOOLEAN NOT NULL,
    counted_by_hash     TEXT NOT NULL,
    counted_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (audit_id, ballot_id_hash)
);

CREATE INDEX IF NOT EXISTS idx_rla_api_audits_election ON rla_api_audits(election_id);
CREATE INDEX IF NOT EXISTS idx_rla_api_samples_audit_round ON rla_api_samples(audit_id, round);

-- Reversible: DROP TABLE IF EXISTS rla_api_counts, rla_api_samples, rla_api_audits CASCADE;
