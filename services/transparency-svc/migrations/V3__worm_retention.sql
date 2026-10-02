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

-- WORM retention on the transparency log core tables (ADR-0017 integrity).
--
-- Even a database superuser path through the app must not be able to edit or
-- remove published leaves, signed tree heads, or witness cosignatures. These
-- triggers make any UPDATE/DELETE a hard error; the app only ever INSERTs.
--
-- Operational escape hatch: an explicit, logged ceremony must DROP and
-- recreate the trigger -- an auditable, deliberate act, never a silent edit.

CREATE OR REPLACE FUNCTION tlog_worm_guard()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'tlog WORM violation: % on % is forbidden (ADR-0017)',
        TG_OP, TG_TABLE_NAME;
END;
$$;

DO $$
DECLARE
    t TEXT;
BEGIN
    FOREACH t IN ARRAY ARRAY['leaves', 'sth', 'witness_cosigs', 'tree_tiles'] LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS worm_%s ON %I', t, t);
        EXECUTE format(
            'CREATE TRIGGER worm_%s BEFORE UPDATE OR DELETE ON %I
             FOR EACH ROW EXECUTE FUNCTION tlog_worm_guard()', t, t);
    END LOOP;
END $$;

COMMENT ON FUNCTION tlog_worm_guard IS
    'Blocks UPDATE/DELETE on append-only transparency log tables';
