-- Participation token lookup log (M13, WP-4.7)
CREATE TABLE IF NOT EXISTS participation_token_lookups (
    lookup_id           TEXT        NOT NULL PRIMARY KEY,  -- UUID7
    token_sha256        TEXT        NOT NULL,
    inclusion_proof_ref TEXT,                              -- reference to tlog entry
    verified_at         TIMESTAMPTZ,
    status              TEXT        NOT NULL DEFAULT 'pending',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_ptl_token_sha256 ON participation_token_lookups (token_sha256);
