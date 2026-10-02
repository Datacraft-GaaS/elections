# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

from __future__ import annotations

AUDIT_SHADOW_DDL_TEMPLATE = """\
-- Audit shadow table for {schema}.{table_name}
CREATE TABLE IF NOT EXISTS {schema}.audit_{table_name}_history (
    id BIGSERIAL PRIMARY KEY,
    operation TEXT NOT NULL,
    old_data JSONB,
    new_data JSONB,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by TEXT
);

-- Trigger function for {schema}.{table_name}
CREATE OR REPLACE FUNCTION {schema}.audit_{table_name}_trigger_fn()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        INSERT INTO {schema}.audit_{table_name}_history
            (operation, old_data, new_data, changed_by)
        VALUES ('DELETE', row_to_json(OLD)::JSONB, NULL, current_user);
        RETURN OLD;
    ELSIF (TG_OP = 'UPDATE') THEN
        INSERT INTO {schema}.audit_{table_name}_history
            (operation, old_data, new_data, changed_by)
        VALUES ('UPDATE', row_to_json(OLD)::JSONB, row_to_json(NEW)::JSONB, current_user);
        RETURN NEW;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO {schema}.audit_{table_name}_history
            (operation, old_data, new_data, changed_by)
        VALUES ('INSERT', NULL, row_to_json(NEW)::JSONB, current_user);
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$;

-- Attach trigger to {schema}.{table_name}
DROP TRIGGER IF EXISTS audit_{table_name}_trigger ON {schema}.{table_name};
CREATE TRIGGER audit_{table_name}_trigger
    AFTER INSERT OR UPDATE OR DELETE ON {schema}.{table_name}
    FOR EACH ROW EXECUTE FUNCTION {schema}.audit_{table_name}_trigger_fn();
"""


def generate_audit_shadow_trigger(table_name: str, schema: str = "public") -> str:
    """
    Returns SQL DDL for a PostgreSQL audit shadow trigger.
    Creates audit_<table_name>_history table with columns:
    id BIGSERIAL, operation TEXT, old_data JSONB, new_data JSONB,
    changed_at TIMESTAMPTZ, changed_by TEXT.
    And a trigger function that writes to it on INSERT/UPDATE/DELETE.
    """
    assert table_name, "table_name must be non-empty"
    assert schema, "schema must be non-empty"
    return AUDIT_SHADOW_DDL_TEMPLATE.format(table_name=table_name, schema=schema)
