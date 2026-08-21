//! Debezium outbox table DDL.

pub const CREATE_OUTBOX_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS outbox (
    id              UUID        PRIMARY KEY,
    aggregate_id    TEXT        NOT NULL,
    aggregate_type  TEXT        NOT NULL,
    topic           TEXT        NOT NULL,
    key             TEXT        NOT NULL,
    payload         JSONB       NOT NULL,
    headers         JSONB       NOT NULL DEFAULT '{}',
    sig             BYTEA,
    hlc             BIGINT      NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at    TIMESTAMPTZ
);
"#;

pub const CREATE_OUTBOX_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS outbox_published_at_idx
    ON outbox (published_at)
    WHERE published_at IS NULL;
"#;
