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

-- Event store for rts-svc (ADR-0013)
CREATE TABLE IF NOT EXISTS eel_events (
    id              UUID PRIMARY KEY,
    stream_id       UUID NOT NULL,
    stream_version  BIGINT NOT NULL,
    event_type      TEXT NOT NULL,
    payload         JSONB NOT NULL,
    metadata        JSONB NOT NULL DEFAULT '{}',
    producer_signature BYTEA,
    hsm_countersignature BYTEA,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    event_time      TIMESTAMPTZ NOT NULL,
    hlc             BIGINT NOT NULL,
    prev_hash       BYTEA,
    UNIQUE (stream_id, stream_version)
);
CREATE INDEX IF NOT EXISTS idx_eel_events_stream ON eel_events (stream_id, stream_version);
CREATE INDEX IF NOT EXISTS idx_eel_events_type ON eel_events (event_type);
CREATE INDEX IF NOT EXISTS idx_eel_events_recorded ON eel_events (recorded_at);

CREATE TABLE IF NOT EXISTS eel_snapshots (
    stream_id       UUID NOT NULL,
    stream_version  BIGINT NOT NULL,
    state           JSONB NOT NULL,
    taken_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (stream_id, stream_version)
);

CREATE TABLE IF NOT EXISTS eel_outbox (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    topic           TEXT NOT NULL,
    payload         JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at    TIMESTAMPTZ,
    attempt_count   INT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_eel_outbox_unpublished ON eel_outbox (created_at) WHERE published_at IS NULL;
