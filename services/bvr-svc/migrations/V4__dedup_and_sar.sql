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

-- Dedup orders and SAR requests
CREATE TABLE IF NOT EXISTS dedup_orders (
    order_id        TEXT        NOT NULL PRIMARY KEY,
    voter_id        TEXT        NOT NULL,
    modality        TEXT        NOT NULL,
    status          TEXT        NOT NULL DEFAULT 'pending',
    investigator_id TEXT,
    voter_ids       JSONB       NOT NULL DEFAULT '[]',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at     TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_dedup_voter    ON dedup_orders (voter_id);
CREATE INDEX IF NOT EXISTS idx_dedup_status   ON dedup_orders (status) WHERE status != 'resolved';

CREATE TABLE IF NOT EXISTS sar_requests (
    sar_id                  TEXT NOT NULL PRIMARY KEY,
    voter_id                TEXT NOT NULL,
    requester_national_id   TEXT NOT NULL,
    purpose                 TEXT NOT NULL DEFAULT 'data_portability',
    status                  TEXT NOT NULL DEFAULT 'pending',
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at             TIMESTAMPTZ,
    resolution_notes        TEXT
);

CREATE INDEX IF NOT EXISTS idx_sar_voter  ON sar_requests (voter_id);
CREATE INDEX IF NOT EXISTS idx_sar_status ON sar_requests (status) WHERE status = 'pending';
