-- Event store for rts-svc (ADR-0013)
CREATE TABLE IF NOT EXISTS eel_events (
    id              UUID PRIMARY KEY,
    stream_id       UUID NOT NULL,
    stream_version  BIGINT NOT NULL,
    event_type      TEXT NOT NULL,
    payload         JSONB NOT NULL,
    metadata        JSONB NOT NULL DEFAULT '{}',
    producer_signature BYTEA,
    hsm_countersignature BYTEA,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    event_time      TIMESTAMPTZ NOT NULL,
    hlc             BIGINT NOT NULL,
    prev_hash       BYTEA,
    UNIQUE (stream_id, stream_version)
);
CREATE INDEX IF NOT EXISTS idx_eel_events_stream ON eel_events (stream_id, stream_version);
CREATE INDEX IF NOT EXISTS idx_eel_events_type ON eel_events (event_type);
CREATE INDEX IF NOT EXISTS idx_eel_events_recorded ON eel_events (recorded_at);

CREATE TABLE IF NOT EXISTS eel_snapshots (
    stream_id       UUID NOT NULL,
    stream_version  BIGINT NOT NULL,
    state           JSONB NOT NULL,
    taken_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (stream_id, stream_version)
);

CREATE TABLE IF NOT EXISTS eel_outbox (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    topic           TEXT NOT NULL,
    payload         JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at    TIMESTAMPTZ,
    attempt_count   INT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_eel_outbox_unpublished ON eel_outbox (created_at) WHERE published_at IS NULL;
