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

-- V055: scope the tamper-alert append-only guard to the evidence columns.
--
-- V051 installed `eel_rfid_append_only_guard()` unconditionally on both
-- `eel_cover_state_transitions` and `eel_cover_tamper_alerts`, copied from
-- V013's reviewer-actions trigger. That pattern is correct for an action log,
-- which is what V013 guards: `eel_reviewer_actions` has no mutable column at
-- all, so refusing every UPDATE costs nothing.
--
-- `eel_cover_tamper_alerts` is not that shape. Six independent artifacts say
-- the row has a lifecycle:
--
--   1. The table's own COMMENT: "tracks ECC dispatch and resolution".
--   2. Four columns that exist only to be written after INSERT --
--      `ecc_dispatched`, `ecc_dispatched_at`, `resolved_at`, `resolution_note`.
--   3. The partial index `eel_cover_tamper_alerts_pending_idx`, defined
--      `WHERE resolved_at IS NULL`. "Pending" is the only definition the schema
--      gives the word, and it is only meaningful if the column changes.
--   4. `rfid-cover-custody/src/repository.rs::resolve_alert`, which issues
--      `UPDATE eel_cover_tamper_alerts SET resolved_at = $2, resolution_note = $3`.
--   5. `rfid-cover-custody/src/api.rs`, which routes
--      `PATCH /v1/rfid/tamper-alerts/{id}/resolve` to that function -- a live
--      endpoint that could only ever answer 500.
--   6. The Django dual-control resolve view, which performs the same UPDATE.
--
-- So the trigger was the outlier, not the writers. Verified against Postgres
-- before the fix: the exact statement in (4) raised
-- `table eel_cover_tamper_alerts is append-only; UPDATE and DELETE are not
-- permitted (ADR-0016)`, and so did the ECC-dispatch UPDATE that sets
-- `ecc_dispatched_at` -- which no code performs today, because the column is
-- unreachable by construction.
--
-- What stays append-only is the evidence: `alert_id`, `tag_id`, `detected_at`,
-- `triggering_read`, `expected_state`, `observed_state`, `severity`. A tamper
-- alert is the record that a seal was broken; rewriting what was detected would
-- destroy the finding. `eel_cover_state_transitions` keeps the unconditional
-- guard, because every column it has is evidence and its BLAKE3 chain covers
-- all of them -- a mutation there breaks the chain, which is the whole point.
--
-- The guard now keys on the *set of changed columns*, so an UPDATE that touches
-- evidence and lifecycle together is refused, and one that touches only
-- lifecycle is permitted. Column allowlist rather than a trigger per column,
-- because a trigger cannot see which columns a statement targeted.
--
-- The error message keeps naming ADR-0016 for continuity with the V051 text,
-- but note that ADR-0016 is "PII classification & retention" and states no
-- append-only rule; the rule for these tables is the table COMMENT plus this
-- migration. Recorded here because the V051 message cites an ADR that does not
-- contain the cited decision, which is how the contradiction went unnoticed.

CREATE OR REPLACE FUNCTION eel_rfid_tamper_alert_guard()
    RETURNS trigger
    LANGUAGE plpgsql AS
$$
DECLARE
    -- The columns a resolution or ECC dispatch may set. Everything else on the
    -- row is the evidence of the tamper event and is frozen.
    mutable CONSTANT text[] := ARRAY[
        'ecc_dispatched',
        'ecc_dispatched_at',
        'resolved_at',
        'resolution_note'
    ];
    changed text[];
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION
            'table % is append-only; DELETE is not permitted',
            TG_TABLE_NAME;
    END IF;

    -- Columns present in the new row whose value differs from the old. A
    -- no-op UPDATE (SET severity = severity) changes nothing and is allowed,
    -- which is the correct reading: it rewrites no evidence.
    SELECT array_agg(n.key ORDER BY n.key)
      INTO changed
      FROM jsonb_each(to_jsonb(NEW)) AS n
     WHERE to_jsonb(NEW) -> n.key IS DISTINCT FROM to_jsonb(OLD) -> n.key;

    IF changed IS NULL THEN
        RETURN NEW;
    END IF;

    IF NOT (changed <@ mutable) THEN
        RAISE EXCEPTION
            'table % is append-only for its evidence columns; UPDATE of % is not permitted (only % may change)',
            TG_TABLE_NAME,
            array_to_string(ARRAY(SELECT unnest(changed) EXCEPT SELECT unnest(mutable)), ', '),
            array_to_string(mutable, ', ');
    END IF;

    RETURN NEW;
END;
$$;

-- Replace V051's unconditional triggers on the alert table only. The
-- state-transitions triggers are left exactly as V051 created them.
DROP TRIGGER IF EXISTS eel_cover_tamper_alerts_no_update ON eel_cover_tamper_alerts;
DROP TRIGGER IF EXISTS eel_cover_tamper_alerts_no_delete ON eel_cover_tamper_alerts;

CREATE TRIGGER eel_cover_tamper_alerts_lifecycle_update
    BEFORE UPDATE ON eel_cover_tamper_alerts
    FOR EACH ROW EXECUTE FUNCTION eel_rfid_tamper_alert_guard();

CREATE TRIGGER eel_cover_tamper_alerts_no_delete
    BEFORE DELETE ON eel_cover_tamper_alerts
    FOR EACH ROW EXECUTE FUNCTION eel_rfid_tamper_alert_guard();

COMMENT ON FUNCTION eel_rfid_tamper_alert_guard() IS
    'ADR-0016 data-class C2: freezes a tamper alert''s evidence columns (alert_id, tag_id, detected_at, triggering_read, expected_state, observed_state, severity) and permits UPDATE only of the lifecycle columns (ecc_dispatched, ecc_dispatched_at, resolved_at, resolution_note). DELETE is always refused. V051 applied the unconditional eel_rfid_append_only_guard() here, which made the resolution columns unreachable and the PATCH /v1/rfid/tamper-alerts/{id}/resolve route a guaranteed 500.';

-- Down (reversible): restore V051's unconditional guard on this table.
-- To rollback: DROP TRIGGER IF EXISTS eel_cover_tamper_alerts_lifecycle_update ON eel_cover_tamper_alerts;
--              DROP TRIGGER IF EXISTS eel_cover_tamper_alerts_no_delete ON eel_cover_tamper_alerts;
--              CREATE TRIGGER eel_cover_tamper_alerts_no_update BEFORE UPDATE ON eel_cover_tamper_alerts FOR EACH ROW EXECUTE FUNCTION eel_rfid_append_only_guard();
--              CREATE TRIGGER eel_cover_tamper_alerts_no_delete BEFORE DELETE ON eel_cover_tamper_alerts FOR EACH ROW EXECUTE FUNCTION eel_rfid_append_only_guard();
--              DROP FUNCTION IF EXISTS eel_rfid_tamper_alert_guard();
