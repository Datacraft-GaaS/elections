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

-- M20 inventory: warehouse, transfer, pallet, and incident domain (WP-4.3).
-- Master / regional / county warehouses; inter-warehouse transfers with
-- departure + arrival; pallet tracking from receipt to polling-station use;
-- break bulk; tamper/incident notes. All rows are data-class C (operational).

-- ---------------------------------------------------------------------------
-- Warehouses
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS warehouses (
    id                  TEXT PRIMARY KEY,
    code                TEXT NOT NULL UNIQUE,
    name                TEXT NOT NULL,
    tier                TEXT NOT NULL CHECK (tier IN ('master', 'regional', 'county')),
    latitude            DOUBLE PRECISION NOT NULL,
    longitude           DOUBLE PRECISION NOT NULL,
    geofence_radius_m   DOUBLE PRECISION NOT NULL CHECK (geofence_radius_m > 0)
);

CREATE INDEX IF NOT EXISTS idx_warehouses_tier ON warehouses (tier);

-- ---------------------------------------------------------------------------
-- Transfers
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS warehouse_transfers (
    id                  TEXT PRIMARY KEY,
    from_warehouse_id   TEXT NOT NULL REFERENCES warehouses (id),
    to_warehouse_id     TEXT NOT NULL REFERENCES warehouses (id),
    vehicle_id          TEXT,
    status              TEXT NOT NULL DEFAULT 'created'
                        CHECK (status IN ('created', 'departed', 'arrived', 'cancelled')),
    expected_pallets    JSONB NOT NULL DEFAULT '[]',
    arrived_pallets     JSONB NOT NULL DEFAULT '[]',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    departed_at         TIMESTAMPTZ,
    arrived_at          TIMESTAMPTZ,
    merkle_node_id      TEXT,
    CHECK (from_warehouse_id <> to_warehouse_id)
);

CREATE INDEX IF NOT EXISTS idx_warehouse_transfers_from ON warehouse_transfers (from_warehouse_id, status);
CREATE INDEX IF NOT EXISTS idx_warehouse_transfers_to   ON warehouse_transfers (to_warehouse_id, status);

-- ---------------------------------------------------------------------------
-- Pallets and break bulk
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS pallets (
    id                  TEXT PRIMARY KEY,
    serial              TEXT NOT NULL UNIQUE,
    warehouse_id        TEXT NOT NULL REFERENCES warehouses (id),
    status              TEXT NOT NULL DEFAULT 'received'
                        CHECK (status IN ('received','in_storage','in_transit',
                                          'break_bulk','dispatched','used')),
    received_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    used_at_station     TEXT,
    merkle_node_id      TEXT
);

CREATE INDEX IF NOT EXISTS idx_pallets_warehouse ON pallets (warehouse_id, status);
CREATE INDEX IF NOT EXISTS idx_pallets_serial    ON pallets (serial);

CREATE TABLE IF NOT EXISTS pallet_items (
    id              TEXT PRIMARY KEY,
    pallet_id       TEXT NOT NULL REFERENCES pallets (id),
    item_type       TEXT NOT NULL CHECK (item_type IN ('ballot_box','ballot_lot','kit','consumable')),
    reference_id    TEXT NOT NULL,
    quantity        BIGINT NOT NULL CHECK (quantity > 0)
);

CREATE INDEX IF NOT EXISTS idx_pallet_items_pallet ON pallet_items (pallet_id);

-- ---------------------------------------------------------------------------
-- Incidents and notes
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS incidents (
    id              TEXT PRIMARY KEY,
    warehouse_id    TEXT REFERENCES warehouses (id),
    pallet_id       TEXT REFERENCES pallets (id),
    transfer_id     TEXT REFERENCES warehouse_transfers (id),
    incident_type   TEXT NOT NULL CHECK (incident_type IN ('tamper','loss','damage','discrepancy','other')),
    severity        TEXT NOT NULL CHECK (severity IN ('P0','P1','P2','P3')),
    summary         TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'open'
                    CHECK (status IN ('open','investigating','resolved')),
    reported_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    reported_by     TEXT NOT NULL,
    resolved_at     TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_incidents_status ON incidents (status, severity);

CREATE TABLE IF NOT EXISTS incident_notes (
    id              TEXT PRIMARY KEY,
    incident_id     TEXT NOT NULL REFERENCES incidents (id),
    author          TEXT NOT NULL,
    note            TEXT NOT NULL,
    at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_incident_notes_incident ON incident_notes (incident_id, at);
