/// DDL for the event store tables. Run once per database / schema.
pub const CREATE_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS eel_events (
    stream_id       UUID        NOT NULL,
    stream_version  BIGINT      NOT NULL,
    id              UUID        NOT NULL,
    event_type      TEXT        NOT NULL,
    payload         JSONB       NOT NULL,
    prev_hash       TEXT,
    hlc             TEXT        NOT NULL,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT eel_events_pkey PRIMARY KEY (stream_id, stream_version)
);

CREATE TABLE IF NOT EXISTS eel_snapshots (
    stream_id       UUID        NOT NULL,
    stream_version  BIGINT      NOT NULL,
    state           JSONB       NOT NULL,
    taken_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT eel_snapshots_pkey PRIMARY KEY (stream_id)
);

CREATE INDEX IF NOT EXISTS eel_events_stream_version_idx
    ON eel_events (stream_id, stream_version);

CREATE INDEX IF NOT EXISTS eel_events_hlc_idx
    ON eel_events (stream_id, hlc);
"#;
