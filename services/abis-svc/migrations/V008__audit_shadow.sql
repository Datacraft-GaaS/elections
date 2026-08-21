-- Audit shadow table: mirrors sensitive operations for T1-audit per M03.
-- Data class D (metadata, no voter PII) per ADR-0016.
CREATE TABLE IF NOT EXISTS eel_abis_audit_shadow (
    audit_id         TEXT        PRIMARY KEY,
    operation        TEXT        NOT NULL,  -- 'match_1n','match_11','adjudication_open','adjudication_resolve','batch_dedup'
    actor_id         TEXT,
    resource_id      TEXT,
    result_code      TEXT,
    client_id        TEXT,
    request_id       TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_eel_abis_audit_created ON eel_abis_audit_shadow(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_eel_abis_audit_op      ON eel_abis_audit_shadow(operation);
