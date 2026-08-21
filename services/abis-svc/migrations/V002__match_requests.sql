-- Match requests: 1:N and 1:1 match request tracking.
-- Data class C per ADR-0016.
CREATE TABLE IF NOT EXISTS eel_match_requests (
    request_id       TEXT        PRIMARY KEY,
    probe_ref        TEXT        NOT NULL,
    modality         TEXT        NOT NULL,
    election_id      TEXT        NOT NULL,
    status           TEXT        NOT NULL DEFAULT 'queued'
                                 CHECK (status IN ('queued','processing','complete','failed')),
    submitted_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at     TIMESTAMPTZ,
    failure_reason   TEXT,
    idempotency_key  TEXT        UNIQUE,
    caller_id        TEXT
);

CREATE INDEX IF NOT EXISTS idx_eel_match_requests_status ON eel_match_requests(status);
CREATE INDEX IF NOT EXISTS idx_eel_match_requests_submitted_at ON eel_match_requests(submitted_at DESC);
