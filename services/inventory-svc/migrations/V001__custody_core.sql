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

-- M20: custody core tables (WP-4.3)
-- All tables are append-only; no UPDATE/DELETE on core fields (FR-M20-002)

CREATE TABLE IF NOT EXISTS assets (
    asset_id        TEXT PRIMARY KEY,
    asset_type      TEXT NOT NULL CHECK (asset_type IN ('kit','ballot','seal','consumable')),
    serial          TEXT NOT NULL,
    model           TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'new'
        CHECK (status IN ('new','in_transit','in_warehouse','provisioned','deployed',
                          'returned','faulty','in_repair','decommissioned','destroyed',
                          'donated','evidentiary_hold')),
    station_code    TEXT,
    last_seen_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- FR-M20-007: every custody event signed by transferor and transferee, timestamped HLC
CREATE TABLE IF NOT EXISTS custody_events (
    event_id        TEXT PRIMARY KEY,
    asset_id        TEXT NOT NULL REFERENCES assets(asset_id),
    from_custodian  TEXT NOT NULL,
    to_custodian    TEXT NOT NULL,
    location        TEXT NOT NULL,
    ts_hlc          BIGINT NOT NULL,
    signature       TEXT NOT NULL,
    seq             BIGINT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- FR-M20-021: seal lifecycle; seal break requires dual-officer HSM auth (FR-M20-022)
CREATE TABLE IF NOT EXISTS seals (
    seal_id         TEXT PRIMARY KEY,
    asset_id        TEXT NOT NULL REFERENCES assets(asset_id),
    seal_type       TEXT NOT NULL,
    affixed_at      TIMESTAMPTZ NOT NULL,
    affixed_by      TEXT NOT NULL,
    broken_at       TIMESTAMPTZ,
    broken_reason   TEXT,
    broken_by       TEXT,
    signature       TEXT NOT NULL
);

-- FR-M20-028: consumables tracked by lot UID, expiry, location, quantity
CREATE TABLE IF NOT EXISTS consumables (
    item_id                 TEXT PRIMARY KEY,
    consumable_type         TEXT NOT NULL,
    quantity                BIGINT NOT NULL DEFAULT 0,
    unit                    TEXT NOT NULL,
    station_code            TEXT NOT NULL,
    replenishment_threshold BIGINT NOT NULL DEFAULT 10
);

-- FR-M20-032: reserve pool tiers; alert when below threshold (FR-M20-035)
CREATE TABLE IF NOT EXISTS reserve_pools (
    pool_id         TEXT PRIMARY KEY,
    scope           TEXT NOT NULL,
    asset_type      TEXT NOT NULL,
    count           BIGINT NOT NULL DEFAULT 0,
    alert_threshold BIGINT NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- FR-M20-036: Merkle root published every 10 min to 5 channels
CREATE TABLE IF NOT EXISTS merkle_checkpoints (
    id              BIGSERIAL PRIMARY KEY,
    root_hash       TEXT NOT NULL,
    window_start    TIMESTAMPTZ NOT NULL,
    window_end      TIMESTAMPTZ NOT NULL,
    event_count     BIGINT NOT NULL DEFAULT 0,
    published_at    TIMESTAMPTZ
);

-- FR-M20-025: kit GPS heartbeats for geofence monitoring
CREATE TABLE IF NOT EXISTS inventory_kit_heartbeats (
    id              BIGSERIAL PRIMARY KEY,
    kit_id          TEXT NOT NULL,
    rfid_tag        TEXT NOT NULL,
    gps_lat         DOUBLE PRECISION NOT NULL,
    gps_lon         DOUBLE PRECISION NOT NULL,
    battery_pct     SMALLINT NOT NULL,
    ts              TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Audit shadow (FR-M20-062: every access logged with actor, purpose, timestamp)
CREATE TABLE IF NOT EXISTS assets_audit (
    audit_id    BIGSERIAL PRIMARY KEY,
    asset_id    TEXT NOT NULL,
    actor_id    TEXT NOT NULL,
    purpose     TEXT NOT NULL,
    action      TEXT NOT NULL,
    ts          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Outbox (transactional outbox pattern; Debezium CDC)
CREATE TABLE IF NOT EXISTS custody_outbox (
    id              BIGSERIAL PRIMARY KEY,
    event_type      TEXT NOT NULL,
    aggregate_id    TEXT NOT NULL,
    payload         JSONB NOT NULL,
    published       BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_custody_events_asset_id  ON custody_events(asset_id);
CREATE INDEX IF NOT EXISTS idx_custody_events_seq       ON custody_events(asset_id, seq);
CREATE INDEX IF NOT EXISTS idx_seals_asset_id           ON seals(asset_id);
CREATE INDEX IF NOT EXISTS idx_inventory_kit_heartbeats_kit_ts    ON inventory_kit_heartbeats(kit_id, ts);
CREATE INDEX IF NOT EXISTS idx_custody_outbox_pending   ON custody_outbox(id) WHERE NOT published;
