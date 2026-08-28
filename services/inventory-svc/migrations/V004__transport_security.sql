-- M20 inventory: vehicle transport and security deployment (WP-4.3).
-- Ballot lots move warehouse -> county -> constituency -> station by vehicle,
-- each load item captured by an RFID or barcode scan; security agents escort
-- vehicles and guard stations. All rows are data-class C (operational).

-- ---------------------------------------------------------------------------
-- Vehicles
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS vehicles (
    id              TEXT PRIMARY KEY,
    registration    TEXT NOT NULL UNIQUE,
    vehicle_type    TEXT NOT NULL CHECK (vehicle_type IN ('truck', 'van', 'air')),
    capacity        BIGINT NOT NULL CHECK (capacity > 0),
    status          TEXT NOT NULL DEFAULT 'available'
                    CHECK (status IN ('available', 'loading', 'dispatched', 'returned'))
);

-- ---------------------------------------------------------------------------
-- Loads and scanned items
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS vehicle_loads (
    id                  TEXT PRIMARY KEY,
    vehicle_id          TEXT NOT NULL REFERENCES vehicles (id),
    destination         TEXT NOT NULL,
    escort_agent_id     TEXT,
    status              TEXT NOT NULL DEFAULT 'loading'
                        CHECK (status IN ('loading', 'dispatched', 'delivered')),
    loaded_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    dispatched_at       TIMESTAMPTZ,
    delivered_at        TIMESTAMPTZ,
    merkle_node_id      TEXT
);

CREATE TABLE IF NOT EXISTS vehicle_load_items (
    id              TEXT PRIMARY KEY,
    load_id         TEXT NOT NULL REFERENCES vehicle_loads (id),
    lot_id          TEXT NOT NULL,
    quantity        BIGINT NOT NULL CHECK (quantity > 0),
    scan_method     TEXT NOT NULL CHECK (scan_method IN ('rfid', 'barcode')),
    scan_value      TEXT NOT NULL,
    scanned_by      TEXT NOT NULL,
    scanned_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (load_id, lot_id)
);

CREATE INDEX IF NOT EXISTS idx_vehicle_loads_vehicle ON vehicle_loads (vehicle_id, status);
CREATE INDEX IF NOT EXISTS idx_vehicle_load_items_load ON vehicle_load_items (load_id);
CREATE INDEX IF NOT EXISTS idx_vehicle_load_items_lot  ON vehicle_load_items (lot_id);

-- ---------------------------------------------------------------------------
-- Security agents and assignments
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS security_agents (
    id              TEXT PRIMARY KEY,
    badge_number    TEXT NOT NULL UNIQUE,
    unit            TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS security_assignments (
    id              TEXT PRIMARY KEY,
    agent_id        TEXT NOT NULL REFERENCES security_agents (id),
    target          TEXT NOT NULL CHECK (target IN ('vehicle', 'station')),
    target_id       TEXT NOT NULL,
    starts_at       TIMESTAMPTZ NOT NULL,
    ends_at         TIMESTAMPTZ NOT NULL,
    status          TEXT NOT NULL DEFAULT 'assigned'
                    CHECK (status IN ('assigned', 'on_duty', 'completed')),
    CHECK (ends_at > starts_at)
);

CREATE INDEX IF NOT EXISTS idx_security_assignments_agent
    ON security_assignments (agent_id, starts_at, ends_at);
CREATE INDEX IF NOT EXISTS idx_security_assignments_target
    ON security_assignments (target, target_id);
