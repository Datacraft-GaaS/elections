-- portal-svc migration V1: cache_control_hints (Refinery managed)
-- Stores per-snapshot cache expiry hints used by portal-api and edge caches.
-- Petition-window rules: T+0..T+7 aggressive, T+8..T+30 relaxed, T+31+ near-static.
-- No PII in this table.

CREATE TABLE IF NOT EXISTS cache_control_hints (
    snapshot_id         UUID        NOT NULL,
    merkle_root         TEXT        NOT NULL,
    published_at        TIMESTAMPTZ NOT NULL,
    max_age_seconds     INTEGER     NOT NULL DEFAULT 30,
    s_maxage_seconds    INTEGER     NOT NULL DEFAULT 30,
    stale_while_revalidate INTEGER  NOT NULL DEFAULT 10,
    petition_day        INTEGER,          -- days since national declaration (NULL if pre-declaration)
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (snapshot_id)
);

CREATE INDEX IF NOT EXISTS idx_cch_published_at ON cache_control_hints (published_at DESC);

COMMENT ON TABLE cache_control_hints IS
  'Per-snapshot cache expiry hints for the edge and portal-api (WP-3.9, FR-M11-024).';
COMMENT ON COLUMN cache_control_hints.petition_day IS
  'Null before declaration; 0 on declaration day; drives TTL rules per ADR-0010.';
