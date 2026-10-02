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

-- V051: RFID ballot-box cover custody tables
-- ADR-0016 data-class C2 (high-value physical asset chain-of-custody)
-- Ref: docs/features/rfid-ballot-box-covers.md, section 3

-- eel_rfid_cover_tags: one row per manufactured cover tag
CREATE TABLE eel_rfid_cover_tags (
    tag_id                  UUID        PRIMARY KEY,
    epc                     BYTEA       NOT NULL UNIQUE,
    manufacturer            TEXT        NOT NULL,
    -- lot_id references eel_printing_lots (deferred FK; that table lives in M39)
    lot_id                  UUID        NOT NULL,
    chip_family             TEXT        NOT NULL,
    tag_secret_hsm_key_id   TEXT        NOT NULL,
    issued_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    paired_with_box         UUID,
    paired_at               TIMESTAMPTZ,
    tamper_state            TEXT        NOT NULL DEFAULT 'intact'
        CHECK (tamper_state IN ('intact','cover_lifted','antenna_broken','clone_detected','killed')),
    tamper_detected_at      TIMESTAMPTZ,
    tamper_detected_by      TEXT,
    retired_at              TIMESTAMPTZ
);

COMMENT ON TABLE eel_rfid_cover_tags
    IS 'ADR-0016 data-class C2: one row per cover RFID tag; bound to a ballot box cover at manufacture and paired to a box body at the printer facility';

CREATE UNIQUE INDEX eel_rfid_cover_tags_epc_idx ON eel_rfid_cover_tags (epc);

-- eel_rfid_cover_reads: every RFID read at any gate or handheld
CREATE TABLE eel_rfid_cover_reads (
    read_id             UUID        PRIMARY KEY,
    tag_id              UUID        NOT NULL REFERENCES eel_rfid_cover_tags (tag_id),
    reader_id           TEXT        NOT NULL,
    reader_type         TEXT        NOT NULL
        CHECK (reader_type IN (
            'warehouse_gate', 'transit_checkpoint', 'station_arrival',
            'kit_handheld', 'ro_desktop', 'archive_gate',
            'airline_cargo', 'mfa_customs'
        )),
    read_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    geo_lat             DOUBLE PRECISION,
    geo_lon             DOUBLE PRECISION,
    geo_accuracy_m      DOUBLE PRECISION,
    tag_challenge       BYTEA,
    tag_response        BYTEA,
    challenge_valid     BOOLEAN     NOT NULL DEFAULT false,
    tamper_indicator    BOOLEAN     NOT NULL DEFAULT false,
    reader_signature    BYTEA,
    custody_event_id    UUID,
    correlation_id      UUID
);

COMMENT ON TABLE eel_rfid_cover_reads
    IS 'ADR-0016 data-class C2: every RFID read event; carries HMAC challenge+response, reader signature, geo, and custody-event linkage';

CREATE INDEX eel_rfid_cover_reads_tag_time_idx
    ON eel_rfid_cover_reads (tag_id, read_at DESC);

CREATE INDEX eel_rfid_cover_reads_tamper_idx
    ON eel_rfid_cover_reads (tag_id)
    WHERE tamper_indicator = true;

CREATE INDEX eel_rfid_cover_reads_reader_time_idx
    ON eel_rfid_cover_reads (reader_id, read_at DESC);

-- eel_cover_state_transitions: append-only lifecycle FSM with per-tag BLAKE3 hash chain
CREATE TABLE eel_cover_state_transitions (
    transition_id       UUID        PRIMARY KEY,
    tag_id              UUID        NOT NULL REFERENCES eel_rfid_cover_tags (tag_id),
    from_state          TEXT        NOT NULL,
    to_state            TEXT        NOT NULL
        CHECK (to_state IN (
            'MANUFACTURED', 'PRINTER', 'WAREHOUSE', 'DISPATCHED', 'IN_TRANSIT',
            'STATION_RECEIVED', 'STATION_OPENED', 'VOTING_ACTIVE', 'STATION_CLOSED',
            'SEALED_RETURN', 'TALLY_ARRIVED', 'COUNT_COMPLETE', 'STORAGE_ARCHIVE',
            'DESTROYED', 'QUARANTINE'
        )),
    triggered_by_read   UUID        REFERENCES eel_rfid_cover_reads (read_id),
    officer_id          UUID,
    officer_svid        TEXT,
    officer_signature   BYTEA,
    prev_hash           BYTEA,
    this_hash           BYTEA       NOT NULL,
    transitioned_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE eel_cover_state_transitions
    IS 'ADR-0016 data-class C2: append-only FSM transitions; each row carries a BLAKE3 hash chain over the full tag lifecycle; silent rewrites break the chain';

CREATE UNIQUE INDEX eel_cover_state_transitions_tag_hash_idx
    ON eel_cover_state_transitions (tag_id, this_hash);

-- eel_cover_tamper_alerts: cover-lift / clone / mismatch alerts with ECC dispatch state
CREATE TABLE eel_cover_tamper_alerts (
    alert_id            UUID        PRIMARY KEY,
    tag_id              UUID        NOT NULL REFERENCES eel_rfid_cover_tags (tag_id),
    detected_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    triggering_read     UUID        REFERENCES eel_rfid_cover_reads (read_id),
    expected_state      TEXT,
    observed_state      TEXT,
    severity            TEXT        NOT NULL DEFAULT 'warn'
        CHECK (severity IN ('info', 'warn', 'critical')),
    ecc_dispatched      BOOLEAN     NOT NULL DEFAULT false,
    ecc_dispatched_at   TIMESTAMPTZ,
    resolved_at         TIMESTAMPTZ,
    resolution_note     TEXT
);

COMMENT ON TABLE eel_cover_tamper_alerts
    IS 'ADR-0016 data-class C2: any cover-lift, clone detection, or EPC mismatch alert; tracks ECC dispatch and resolution';

CREATE INDEX eel_cover_tamper_alerts_pending_idx
    ON eel_cover_tamper_alerts (tag_id)
    WHERE resolved_at IS NULL;

-- Append-only enforcement (mirrors the reviewer-actions pattern from V013)
CREATE OR REPLACE FUNCTION eel_rfid_append_only_guard()
    RETURNS trigger
    LANGUAGE plpgsql AS
$$
BEGIN
    RAISE EXCEPTION
        'table % is append-only; UPDATE and DELETE are not permitted (ADR-0016)',
        TG_TABLE_NAME;
END;
$$;

CREATE TRIGGER eel_cover_state_transitions_no_update
    BEFORE UPDATE ON eel_cover_state_transitions
    FOR EACH ROW EXECUTE FUNCTION eel_rfid_append_only_guard();

CREATE TRIGGER eel_cover_state_transitions_no_delete
    BEFORE DELETE ON eel_cover_state_transitions
    FOR EACH ROW EXECUTE FUNCTION eel_rfid_append_only_guard();

CREATE TRIGGER eel_cover_tamper_alerts_no_update
    BEFORE UPDATE ON eel_cover_tamper_alerts
    FOR EACH ROW EXECUTE FUNCTION eel_rfid_append_only_guard();

CREATE TRIGGER eel_cover_tamper_alerts_no_delete
    BEFORE DELETE ON eel_cover_tamper_alerts
    FOR EACH ROW EXECUTE FUNCTION eel_rfid_append_only_guard();
