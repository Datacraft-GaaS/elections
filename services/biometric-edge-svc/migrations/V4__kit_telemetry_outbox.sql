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

-- Migration V4: outbox table for kit events (WP-3.8)
-- Follows eel-outbox-rs schema convention.
CREATE TABLE edge_outbox (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type   TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    payload      JSONB NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at TIMESTAMPTZ,
    attempts     SMALLINT NOT NULL DEFAULT 0
);

CREATE INDEX edge_outbox_unpublished
    ON edge_outbox (created_at)
    WHERE published_at IS NULL;

COMMENT ON TABLE edge_outbox IS
    'Transactional outbox for edge events: kit_heartbeat.v1, kit_distress.v1, kit_ota_delivered.v1, kit_bvr_enrolled.v1 (WP-3.8).';

-- Audit shadow table for immutable append-only audit log.
CREATE TABLE edge_audit (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type   TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    payload      JSONB NOT NULL,
    actor_svid   TEXT,
    recorded_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE edge_audit IS 'Immutable audit shadow for edge API events (WP-3.8).';
