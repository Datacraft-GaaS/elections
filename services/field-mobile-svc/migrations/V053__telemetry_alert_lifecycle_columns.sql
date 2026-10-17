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

-- V053: scope the telemetry-alert append-only guard to the evidence columns.
--
-- V052 applied `eel_telemetry_append_only_guard()` unconditionally to
-- `eel_telemetry_signals`, `eel_telemetry_alerts` and `eel_telemetry_snapshots`.
-- Two of those three are correctly guarded; the alert table is not.
--
-- Correctly guarded, and left alone:
--
--   * `eel_telemetry_signals` -- the append-only signal stream, carrying
--     `prev_hash`/`this_hash`. Every column is evidence and no code updates it.
--   * `eel_telemetry_snapshots` -- daily Merkle anchors (`device_merkle_root`,
--     `tlog_leaf_index`, `tlog_root_hash`, `inclusion_proof`). An anchor that
--     can be rewritten proves nothing.
--
-- Wrongly guarded:
--
--   * `eel_telemetry_alerts` -- its own COMMENT says "append-only alert
--     records; resolution recorded via resolved_at column", which is a
--     contradiction in one sentence: the resolution is recorded by an UPDATE,
--     and the trigger refuses every UPDATE. Four columns exist only to be
--     written after INSERT (`ecc_dispatched`, `ecc_dispatched_at`,
--     `resolved_at`, `resolution_note`); the partial index
--     `eel_telemetry_alerts_unresolved` is defined `WHERE resolved_at IS NULL`;
--     and `telemetry-ingest/src/repository.rs::resolve_alert` issues
--     `UPDATE eel_telemetry_alerts SET resolved_at = $1, resolution_note = $2`.
--
-- Verified against Postgres before the fix: that exact statement raised
-- `table eel_telemetry_alerts is append-only; UPDATE and DELETE are not
-- permitted (ADR-0016)`.
--
-- Same repair as inventory-svc V055, and the same reasoning: freeze the
-- evidence, permit the lifecycle. See that migration for the fuller note on the
-- ADR-0016 citation, which does not state an append-only rule.

CREATE OR REPLACE FUNCTION eel_telemetry_alert_guard()
    RETURNS trigger
    LANGUAGE plpgsql AS
$$
DECLARE
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

-- Replace V052's unconditional triggers on the alert table only. Signals and
-- snapshots keep the unconditional guard.
DROP TRIGGER IF EXISTS eel_telemetry_alerts_no_update ON eel_telemetry_alerts;
DROP TRIGGER IF EXISTS eel_telemetry_alerts_no_delete ON eel_telemetry_alerts;

CREATE TRIGGER eel_telemetry_alerts_lifecycle_update
    BEFORE UPDATE ON eel_telemetry_alerts
    FOR EACH ROW EXECUTE FUNCTION eel_telemetry_alert_guard();

CREATE TRIGGER eel_telemetry_alerts_no_delete
    BEFORE DELETE ON eel_telemetry_alerts
    FOR EACH ROW EXECUTE FUNCTION eel_telemetry_alert_guard();

COMMENT ON FUNCTION eel_telemetry_alert_guard() IS
    'ADR-0016 data-class C1: freezes a telemetry alert''s evidence columns (alert_id, device_id, class, severity, detected_at, triggering_signal, payload) and permits UPDATE only of the lifecycle columns (ecc_dispatched, ecc_dispatched_at, resolved_at, resolution_note). DELETE is always refused. V052 applied the unconditional eel_telemetry_append_only_guard() here, contradicting the table COMMENT it wrote in the same file.';

-- Down (reversible): restore V052's unconditional guard on this table.
-- To rollback: DROP TRIGGER IF EXISTS eel_telemetry_alerts_lifecycle_update ON eel_telemetry_alerts;
--              DROP TRIGGER IF EXISTS eel_telemetry_alerts_no_delete ON eel_telemetry_alerts;
--              CREATE TRIGGER eel_telemetry_alerts_no_update BEFORE UPDATE ON eel_telemetry_alerts FOR EACH ROW EXECUTE FUNCTION eel_telemetry_append_only_guard();
--              CREATE TRIGGER eel_telemetry_alerts_no_delete BEFORE DELETE ON eel_telemetry_alerts FOR EACH ROW EXECUTE FUNCTION eel_telemetry_append_only_guard();
--              DROP FUNCTION IF EXISTS eel_telemetry_alert_guard();
