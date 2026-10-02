-- Copyright (c) 2026 Nyimbi Odero
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.

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
