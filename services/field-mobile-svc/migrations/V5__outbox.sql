-- Transactional outbox for verifier events (eel-outbox pattern)
CREATE TABLE IF NOT EXISTS verifier_outbox (
    outbox_id    TEXT        NOT NULL PRIMARY KEY,  -- UUID7
    topic        TEXT        NOT NULL,               -- eel.verifier.audit.v1 etc.
    event_type   TEXT        NOT NULL,
    payload      JSONB       NOT NULL,
    published    BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_outbox_unpublished ON verifier_outbox (created_at) WHERE NOT published;
