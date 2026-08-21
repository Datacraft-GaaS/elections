-- M64: voter list inspection, objections, auto-referral initial schema

CREATE TABLE eel_roll_snapshots (
    id              UUID PRIMARY KEY,
    snapshot_hash   TEXT NOT NULL,
    taken_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    station_count   INTEGER NOT NULL,
    voter_count     BIGINT NOT NULL
);

CREATE TABLE eel_objections (
    id              UUID PRIMARY KEY,
    voter_id_hash   TEXT NOT NULL,
    station_id      TEXT NOT NULL,
    reason          TEXT NOT NULL,
    filed_by        UUID NOT NULL,
    filed_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status          TEXT NOT NULL DEFAULT 'filed'
);

CREATE INDEX idx_eel_objections_station ON eel_objections (station_id);

CREATE TABLE eel_correction_claims (
    id                          UUID PRIMARY KEY,
    voter_id_hash               TEXT NOT NULL,
    claimed_by_biometric_hash   TEXT NOT NULL,
    correction_type             TEXT NOT NULL,
    evidence_refs               TEXT[] NOT NULL DEFAULT '{}',
    status                      TEXT NOT NULL DEFAULT 'pending',
    filed_at                    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE eel_determinations (
    id          UUID PRIMARY KEY,
    claim_id    UUID NOT NULL REFERENCES eel_correction_claims (id),
    decision    TEXT NOT NULL,
    decided_by  UUID NOT NULL,
    decided_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE eel_auto_referral_packets (
    id                  UUID PRIMARY KEY,
    correction_claim_id UUID NOT NULL REFERENCES eel_correction_claims (id),
    ecourt_ref          TEXT NOT NULL,
    referred_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Idempotency: one referral per claim
    UNIQUE (correction_claim_id)
);
