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

-- Ballot-serial chain tables (WP-3.3, M63).
-- Append-only; compensating events used for corrections (FR-M63-033).

CREATE TABLE IF NOT EXISTS serial_assignments (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    lot_id           TEXT        NOT NULL,
    station_code     TEXT        NOT NULL,
    serial_low       BIGINT      NOT NULL,
    serial_high      BIGINT      NOT NULL CHECK (serial_high >= serial_low),
    assigned_at      TIMESTAMPTZ NOT NULL,
    assigned_by      TEXT        NOT NULL,
    manifest_ref     TEXT        NOT NULL,
    manifest_sig     TEXT        NOT NULL,
    stream_version   BIGINT      NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_serial_assignment_station
    ON serial_assignments (station_code);

CREATE TABLE IF NOT EXISTS ballot_issuances (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id    UUID        NOT NULL REFERENCES serial_assignments(id),
    clerk_id         TEXT        NOT NULL,
    received_at      TIMESTAMPTZ NOT NULL,
    receipt_hash     TEXT        NOT NULL,
    stream_version   BIGINT      NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ballot_spoils (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    station_code     TEXT        NOT NULL,
    spoilt_serial    BIGINT      NOT NULL,
    replacement_serial BIGINT,
    reason           TEXT        NOT NULL,
    recorded_at      TIMESTAMPTZ NOT NULL,
    clerk_id         TEXT        NOT NULL,
    stream_version   BIGINT      NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_ballot_spoils_station
    ON ballot_spoils (station_code);

CREATE TABLE IF NOT EXISTS reconciliations (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    station_code     TEXT        NOT NULL,
    election_id      TEXT        NOT NULL,
    issued_count     BIGINT      NOT NULL DEFAULT 0,
    spoilt_count     BIGINT      NOT NULL DEFAULT 0,
    cast_count       BIGINT      NOT NULL DEFAULT 0,
    unaccounted_count BIGINT     NOT NULL DEFAULT 0,
    status           TEXT        NOT NULL DEFAULT 'pending'
                                 CHECK (status IN ('pending','held','cleared','flagged')),
    computed_at      TIMESTAMPTZ,
    stream_version   BIGINT      NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS discrepancy_holds (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    reconciliation_id UUID       NOT NULL REFERENCES reconciliations(id),
    flag_reason      TEXT        NOT NULL,
    raised_at        TIMESTAMPTZ NOT NULL,
    resolved_at      TIMESTAMPTZ,
    resolved_by_po   TEXT,
    resolved_by_cro  TEXT,
    resolve_reason   TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Audit shadow: one row per ballot-serial event.
CREATE TABLE IF NOT EXISTS ballot_serial_audit_shadow (
    id             BIGSERIAL   PRIMARY KEY,
    station_code   TEXT        NOT NULL,
    event_type     TEXT        NOT NULL,
    actor_id       TEXT        NOT NULL,
    hlc_ts         TIMESTAMPTZ NOT NULL,
    payload_hash   TEXT        NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_bs_audit_station
    ON ballot_serial_audit_shadow (station_code, hlc_ts);

-- Outbox shared with the workspace (exactly-once delivery, FR-M63-038).
CREATE TABLE IF NOT EXISTS ballot_serial_outbox (
    id               BIGSERIAL   PRIMARY KEY,
    aggregate_type   TEXT        NOT NULL,
    aggregate_id     UUID        NOT NULL,
    event_type       TEXT        NOT NULL,
    payload          JSONB       NOT NULL,
    topic            TEXT        NOT NULL,
    published        BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_bs_outbox_unpublished
    ON ballot_serial_outbox (published, created_at)
    WHERE published = FALSE;
