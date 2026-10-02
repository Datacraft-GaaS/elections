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
