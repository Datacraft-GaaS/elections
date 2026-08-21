-- portal-svc migration V3: viewer_session_metrics (Refinery managed)
-- Aggregate counters only: no PII, no individual session identifiers.
-- Used for Grafana dashboard (FR-M11-039) and SLO burn-rate alerts (FR-M11-040).

CREATE TABLE IF NOT EXISTS viewer_session_metrics (
    bucket_at           TIMESTAMPTZ NOT NULL,   -- truncated to minute
    endpoint            TEXT        NOT NULL,
    http_status_class   TEXT        NOT NULL,   -- '2xx'|'3xx'|'4xx'|'5xx'
    request_count       BIGINT      NOT NULL DEFAULT 0,
    p95_ms              INTEGER,
    p99_ms              INTEGER,
    cache_hits          BIGINT      NOT NULL DEFAULT 0,
    cache_misses        BIGINT      NOT NULL DEFAULT 0,
    PRIMARY KEY (bucket_at, endpoint, http_status_class)
);

CREATE INDEX IF NOT EXISTS idx_vsm_bucket ON viewer_session_metrics (bucket_at DESC);

COMMENT ON TABLE viewer_session_metrics IS
  'Aggregate per-minute request counters for portal SLO monitoring (WP-3.9).
   No PII. No IP addresses. No individual session identifiers.';
