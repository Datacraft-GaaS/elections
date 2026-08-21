-- Projection views for rts-svc read models
CREATE TABLE IF NOT EXISTS form34a_view (
    form_id             UUID PRIMARY KEY,
    stream_id           UUID NOT NULL UNIQUE,
    stream_version      BIGINT NOT NULL,
    station_code        TEXT NOT NULL,
    constituency_code   TEXT NOT NULL,
    county_code         TEXT NOT NULL,
    registered_voters   INT NOT NULL DEFAULT 0,
    votes_cast          INT NOT NULL DEFAULT 0,
    valid_votes         INT NOT NULL DEFAULT 0,
    rejected_votes      INT NOT NULL DEFAULT 0,
    spoilt_ballots      INT NOT NULL DEFAULT 0,
    candidate_totals    JSONB NOT NULL DEFAULT '[]',
    status              TEXT NOT NULL DEFAULT 'draft',
    raw_scan_ref        TEXT,
    annotations         JSONB NOT NULL DEFAULT '[]',
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_form34a_constituency ON form34a_view (constituency_code);
CREATE INDEX IF NOT EXISTS idx_form34a_status ON form34a_view (status);

CREATE TABLE IF NOT EXISTS form34b_view (
    form_id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id           UUID NOT NULL UNIQUE,
    stream_version      BIGINT NOT NULL,
    constituency_code   TEXT NOT NULL UNIQUE,
    county_code         TEXT NOT NULL,
    compiled_stations   JSONB NOT NULL DEFAULT '[]',
    total_registered    BIGINT NOT NULL DEFAULT 0,
    total_valid         BIGINT NOT NULL DEFAULT 0,
    total_rejected      BIGINT NOT NULL DEFAULT 0,
    total_spoilt        BIGINT NOT NULL DEFAULT 0,
    candidate_totals    JSONB NOT NULL DEFAULT '[]',
    status              TEXT NOT NULL DEFAULT 'pending',
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS form34c_view (
    form_id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id               UUID NOT NULL UNIQUE,
    stream_version          BIGINT NOT NULL,
    compiled_constituencies JSONB NOT NULL DEFAULT '[]',
    total_registered        BIGINT NOT NULL DEFAULT 0,
    total_valid             BIGINT NOT NULL DEFAULT 0,
    total_rejected          BIGINT NOT NULL DEFAULT 0,
    total_spoilt            BIGINT NOT NULL DEFAULT 0,
    candidate_totals        JSONB NOT NULL DEFAULT '[]',
    merkle_root             TEXT,
    status                  TEXT NOT NULL DEFAULT 'pending',
    published_at            TIMESTAMPTZ,
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS publication_snapshots (
    snapshot_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    form34c_id      UUID REFERENCES form34c_view(form_id),
    merkle_root     TEXT NOT NULL,
    signed_url      TEXT,
    tlog_leaf_index BIGINT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
