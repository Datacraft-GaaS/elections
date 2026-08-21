-- Verification queries audit log (M13, WP-4.7)
-- Tier C: channel, query_type, result_status logged; no PII; token_sha256 = hashed only
CREATE TABLE IF NOT EXISTS verification_queries (
    query_id           TEXT        NOT NULL PRIMARY KEY,  -- UUID7
    channel            TEXT        NOT NULL,               -- flutter|pwa|ussd|whatsapp|api
    actor_session_hash TEXT        NOT NULL,               -- hashed, no PII
    query_type         TEXT        NOT NULL,               -- token|results|proof|civic
    result_status      TEXT        NOT NULL,               -- verified|unverifiable|not_found|error
    token_sha256       TEXT,                               -- hashed ballot token only
    ip_prefix          TEXT,                               -- /24 for IPv4, /48 for IPv6 (EC-SEC-09)
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_vq_created_at ON verification_queries (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_vq_channel    ON verification_queries (channel);
