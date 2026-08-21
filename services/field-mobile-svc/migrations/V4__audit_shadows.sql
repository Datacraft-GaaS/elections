-- Audit shadow tables for mutable verification records (ADR-0013)
CREATE TABLE IF NOT EXISTS audit_verification_queries_history (
    history_id   TEXT        NOT NULL PRIMARY KEY,
    query_id     TEXT        NOT NULL,
    operation    TEXT        NOT NULL,  -- INSERT|UPDATE|DELETE
    changed_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    row_snapshot JSONB       NOT NULL   -- no PII; encrypted where needed
);

CREATE TABLE IF NOT EXISTS audit_grievance_intake_history (
    history_id   TEXT        NOT NULL PRIMARY KEY,
    grievance_id TEXT        NOT NULL,
    operation    TEXT        NOT NULL,
    changed_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    row_snapshot JSONB       NOT NULL
);
