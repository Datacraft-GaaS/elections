-- Transactional outbox for event publishing per ADR-0015.
-- Data class D per ADR-0016.
CREATE TABLE IF NOT EXISTS eel_abis_outbox (
    id               BIGSERIAL   PRIMARY KEY,
    event_type       TEXT        NOT NULL,
    event_id         TEXT        NOT NULL UNIQUE,
    topic            TEXT        NOT NULL,
    payload          JSONB       NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at     TIMESTAMPTZ,
    retry_count      INT         NOT NULL DEFAULT 0,
    last_error       TEXT
);

CREATE INDEX IF NOT EXISTS idx_eel_abis_outbox_unpublished ON eel_abis_outbox(created_at) WHERE published_at IS NULL;
