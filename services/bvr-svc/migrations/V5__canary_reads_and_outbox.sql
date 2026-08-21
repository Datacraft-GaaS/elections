-- Canary read log (EC-SEC-16)
CREATE TABLE IF NOT EXISTS canary_reads (
    id              TEXT        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    voter_id        TEXT        NOT NULL,
    accessed_by     TEXT        NOT NULL,
    path            TEXT        NOT NULL,
    accessed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_canary_voter ON canary_reads (voter_id);
CREATE INDEX IF NOT EXISTS idx_canary_when  ON canary_reads (accessed_at DESC);

-- Transactional outbox (eel-outbox-rs schema expected by Debezium)
CREATE TABLE IF NOT EXISTS outbox (
    id              TEXT        NOT NULL PRIMARY KEY,
    event_type      TEXT        NOT NULL,
    source          TEXT        NOT NULL,
    subject         TEXT,
    time            TIMESTAMPTZ NOT NULL,
    eel_stream_id   TEXT        NOT NULL,
    eel_seq         BIGINT      NOT NULL,
    eel_hlc         BIGINT      NOT NULL,
    eel_prev_hash   TEXT,
    eel_correlation_id TEXT,
    eel_causation_id   TEXT,
    eel_producer_key_id TEXT,
    eel_signature   BYTEA,
    eel_election_id TEXT,
    eel_data_class  TEXT,
    data            JSONB       NOT NULL,
    topic           TEXT        NOT NULL,
    kafka_key       TEXT        NOT NULL,
    aggregate_type  TEXT        NOT NULL,
    aggregate_id    TEXT        NOT NULL,
    published_at    TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_outbox_unpublished ON outbox (id) WHERE published_at IS NULL;

-- audit_events generic table
CREATE TABLE IF NOT EXISTS audit_events (
    event_id    TEXT        NOT NULL PRIMARY KEY,
    actor       TEXT        NOT NULL,
    action      TEXT        NOT NULL,
    resource    TEXT        NOT NULL,
    ts          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    signed_by   TEXT        NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_audit_ts ON audit_events (ts DESC);
