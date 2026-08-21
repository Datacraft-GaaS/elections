-- USSD session state (ephemeral, 30-minute TTL)
CREATE TABLE IF NOT EXISTS ussd_sessions (
    session_id TEXT        NOT NULL PRIMARY KEY,
    phone_hash TEXT        NOT NULL,    -- hashed phone; no PII
    step       INTEGER     NOT NULL DEFAULT 1,
    context    JSONB       NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '30 minutes'
);
CREATE INDEX IF NOT EXISTS idx_ussd_expires ON ussd_sessions (expires_at);
