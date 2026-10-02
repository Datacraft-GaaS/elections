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

-- M39: ballot printing chain tables (WP-4.3)
-- Ballot chain tampering must be computationally detectable (FR-M39-036)

-- FR-M39-001: dual-signed print order, anchored in Merkle chain before printer receives it
CREATE TABLE IF NOT EXISTS print_orders (
    id                                  TEXT PRIMARY KEY,
    election_id                         TEXT NOT NULL,
    contest_id                          TEXT NOT NULL,
    constituency_code                   TEXT NOT NULL,
    quantity_authorised                 BIGINT NOT NULL,
    serial_range_start                  BIGINT NOT NULL,
    serial_range_end                    BIGINT NOT NULL,
    paper_spec_hash                     TEXT NOT NULL,
    issued_by_officer_id                TEXT NOT NULL,
    countersigned_by_commissioner_id    TEXT NOT NULL,
    issued_at                           TIMESTAMPTZ NOT NULL,
    gazette_notice_id                   TEXT,
    status                              TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending','dual_signed','transmitted','printed',
                          'in_transit','at_warehouse','allocated','at_station','reconciled')),
    merkle_node_id                      TEXT
);

-- FR-M39-003: printer MES submits lot data; reconcile against print order
CREATE TABLE IF NOT EXISTS lot_data (
    id                      TEXT PRIMARY KEY,
    print_order_id          TEXT NOT NULL REFERENCES print_orders(id),
    printer_facility_id     TEXT NOT NULL,
    serial_range_start      BIGINT NOT NULL,
    serial_range_end        BIGINT NOT NULL,
    quantity_printed        BIGINT NOT NULL,
    quantity_rejected       BIGINT NOT NULL DEFAULT 0,
    rfid_chip_id_list_hash  TEXT NOT NULL,
    print_date              TIMESTAMPTZ NOT NULL,
    received_at             TIMESTAMPTZ,
    variance_pct            DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    variance_alert_sent     BOOLEAN NOT NULL DEFAULT false,
    merkle_node_id          TEXT
);

-- FR-M39-005: custody events at each stage (port/warehouse/constituency/station)
CREATE TABLE IF NOT EXISTS print_custody_events (
    id                  TEXT PRIMARY KEY,
    lot_id              TEXT NOT NULL REFERENCES lot_data(id),
    event_type          TEXT NOT NULL
        CHECK (event_type IN ('port_clearance','warehouse_receipt','constituency_allocation',
                              'station_receipt','station_close_return','rfid_spot_check')),
    location_code       TEXT NOT NULL,
    officer_id          TEXT NOT NULL,
    witness_id          TEXT,
    quantity_scanned    BIGINT NOT NULL DEFAULT 0,
    rfid_match_count    BIGINT NOT NULL DEFAULT 0,
    rfid_mismatch_count BIGINT NOT NULL DEFAULT 0,
    seal_number         TEXT,
    vehicle_registration TEXT,
    scanned_at          TIMESTAMPTZ NOT NULL,
    merkle_node_id      TEXT
);

-- FR-M39-010: station end-of-day reconciliation
CREATE TABLE IF NOT EXISTS station_reconciliations (
    id                  TEXT PRIMARY KEY,
    station_code        TEXT NOT NULL,
    election_id         TEXT NOT NULL,
    ballots_received    BIGINT NOT NULL DEFAULT 0,
    ballots_issued      BIGINT NOT NULL DEFAULT 0,
    ballots_unused      BIGINT NOT NULL DEFAULT 0,
    ballots_spoilt      BIGINT NOT NULL DEFAULT 0,
    discrepancy         BIGINT NOT NULL DEFAULT 0,
    discrepancy_flag    BOOLEAN NOT NULL DEFAULT false,
    reconciled_at       TIMESTAMPTZ NOT NULL,
    reconciled_by       TEXT NOT NULL,
    merkle_node_id      TEXT
);

-- Audit shadow
CREATE TABLE IF NOT EXISTS print_orders_audit (
    audit_id        BIGSERIAL PRIMARY KEY,
    print_order_id  TEXT NOT NULL,
    actor_id        TEXT NOT NULL,
    action          TEXT NOT NULL,
    ts              TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Outbox
CREATE TABLE IF NOT EXISTS printing_outbox (
    id              BIGSERIAL PRIMARY KEY,
    event_type      TEXT NOT NULL,
    aggregate_id    TEXT NOT NULL,
    payload         JSONB NOT NULL,
    published       BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_lot_data_print_order      ON lot_data(print_order_id);
CREATE INDEX IF NOT EXISTS idx_print_custody_lot         ON print_custody_events(lot_id);
CREATE INDEX IF NOT EXISTS idx_print_custody_location    ON print_custody_events(location_code);
CREATE INDEX IF NOT EXISTS idx_station_recon_station     ON station_reconciliations(station_code, election_id);
CREATE INDEX IF NOT EXISTS idx_printing_outbox_pending   ON printing_outbox(id) WHERE NOT published;
