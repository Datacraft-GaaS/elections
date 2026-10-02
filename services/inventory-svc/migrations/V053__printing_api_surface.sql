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

-- M39 printing API surface — postgres for public contract (no in-memory)
-- Replaces SharedMap<Uuid, PrintOrder> in printing/http.rs AppState.

CREATE TABLE IF NOT EXISTS printing_api_orders (
    order_id            UUID PRIMARY KEY,
    election_id         UUID NOT NULL,
    position_code       TEXT NOT NULL,
    constituency_code   TEXT NOT NULL,
    ballot_quantity     INTEGER NOT NULL CHECK (ballot_quantity > 0),
    proof_id            UUID NOT NULL,
    security_features   JSONB NOT NULL DEFAULT '[]'::jsonb,
    status              TEXT NOT NULL DEFAULT 'ordered' CHECK (status IN ('ordered','qc','delivered','rejected')),
    printer_reference   TEXT NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS printing_api_deliveries (
    delivery_id         UUID PRIMARY KEY,
    order_id            UUID NOT NULL REFERENCES printing_api_orders(order_id) ON DELETE CASCADE,
    delivered_quantity  INTEGER NOT NULL,
    serial_range_start  TEXT NOT NULL,
    serial_range_end    TEXT NOT NULL,
    received_by_hash    TEXT NOT NULL,
    received_at         TIMESTAMPTZ NOT NULL,
    status              TEXT NOT NULL DEFAULT 'received',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS printing_api_audit (
    event_id        UUID PRIMARY KEY,
    event_type      TEXT NOT NULL,
    order_id        UUID REFERENCES printing_api_orders(order_id) ON DELETE SET NULL,
    actor_role      TEXT,
    payload_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_printing_api_orders_election ON printing_api_orders(election_id);
CREATE INDEX IF NOT EXISTS idx_printing_api_deliveries_order ON printing_api_deliveries(order_id);

-- Reversible: DROP TABLE IF EXISTS printing_api_audit, printing_api_deliveries, printing_api_orders CASCADE;
