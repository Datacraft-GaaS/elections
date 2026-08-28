-- bvi-svc bootstrap: core extensions and audit shadow infrastructure.
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Audit shadow trigger function (shared by all tables).
CREATE OR REPLACE FUNCTION audit_shadow_trigger() RETURNS TRIGGER AS $$
BEGIN
  INSERT INTO audit.shadow_log (table_name, operation, old_row, new_row, changed_at)
  VALUES (TG_TABLE_NAME, TG_OP, to_jsonb(OLD), to_jsonb(NEW), now());
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE SCHEMA IF NOT EXISTS audit;

CREATE TABLE IF NOT EXISTS audit.shadow_log (
  id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  table_name  TEXT        NOT NULL,
  operation   TEXT        NOT NULL,
  old_row     JSONB,
  new_row     JSONB,
  changed_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
