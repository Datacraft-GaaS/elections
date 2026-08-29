-- Inventory shortfalls — postgres for public contract (no OnceLock)
-- Replaces static Mutex<HashMap<String, Shortfall>> in inventory-api/spec.rs

CREATE TABLE IF NOT EXISTS inventory_shortfalls (
    shortfall_id        TEXT PRIMARY KEY,
    constituency_code   TEXT NOT NULL,
    item                TEXT NOT NULL,
    quantity            INTEGER NOT NULL CHECK (quantity > 0),
    reported_by_hash    TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'reported' CHECK (status IN ('reported','acknowledged','fulfilled','rejected')),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    request_json        JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_inventory_shortfalls_constituency ON inventory_shortfalls(constituency_code);
CREATE INDEX IF NOT EXISTS idx_inventory_shortfalls_created ON inventory_shortfalls(created_at);

-- Reversible: DROP TABLE IF EXISTS inventory_shortfalls CASCADE;
