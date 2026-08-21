-- Dedup orders and SAR requests
CREATE TABLE IF NOT EXISTS dedup_orders (
    order_id        TEXT        NOT NULL PRIMARY KEY,
    voter_id        TEXT        NOT NULL,
    modality        TEXT        NOT NULL,
    status          TEXT        NOT NULL DEFAULT 'pending',
    investigator_id TEXT,
    voter_ids       JSONB       NOT NULL DEFAULT '[]',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at     TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_dedup_voter    ON dedup_orders (voter_id);
CREATE INDEX IF NOT EXISTS idx_dedup_status   ON dedup_orders (status) WHERE status != 'resolved';

CREATE TABLE IF NOT EXISTS sar_requests (
    sar_id                  TEXT NOT NULL PRIMARY KEY,
    voter_id                TEXT NOT NULL,
    requester_national_id   TEXT NOT NULL,
    purpose                 TEXT NOT NULL DEFAULT 'data_portability',
    status                  TEXT NOT NULL DEFAULT 'pending',
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at             TIMESTAMPTZ,
    resolution_notes        TEXT
);

CREATE INDEX IF NOT EXISTS idx_sar_voter  ON sar_requests (voter_id);
CREATE INDEX IF NOT EXISTS idx_sar_status ON sar_requests (status) WHERE status = 'pending';
