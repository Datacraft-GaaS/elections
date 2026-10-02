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

-- Audit shadow tables (trigger-populated) per ADR-0013
CREATE SCHEMA IF NOT EXISTS audit;

CREATE TABLE IF NOT EXISTS audit.form34a_view_history (
    LIKE form34a_view INCLUDING ALL,
    _op   TEXT NOT NULL,
    _at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    _by   TEXT
);

CREATE OR REPLACE FUNCTION audit.log_form34a_changes() RETURNS trigger AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        INSERT INTO audit.form34a_view_history SELECT OLD.*, 'D', now(), current_user;
    ELSE
        INSERT INTO audit.form34a_view_history SELECT NEW.*, TG_OP, now(), current_user;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER form34a_audit_trigger
AFTER INSERT OR UPDATE OR DELETE ON form34a_view
FOR EACH ROW EXECUTE FUNCTION audit.log_form34a_changes();
