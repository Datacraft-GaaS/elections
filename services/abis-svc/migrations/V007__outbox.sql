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

-- Transactional outbox for event publishing per ADR-0015.
-- Data class D per ADR-0016.
CREATE TABLE IF NOT EXISTS eel_abis_outbox (
    id               BIGSERIAL   PRIMARY KEY,
    event_type       TEXT        NOT NULL,
    event_id         TEXT        NOT NULL UNIQUE,
    topic            TEXT        NOT NULL,
    payload          JSONB       NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at     TIMESTAMPTZ,
    retry_count      INT         NOT NULL DEFAULT 0,
    last_error       TEXT
);

CREATE INDEX IF NOT EXISTS idx_eel_abis_outbox_unpublished ON eel_abis_outbox(created_at) WHERE published_at IS NULL;
