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

-- V053: card production batches (M53 issuance)
-- Columns mirror voter-card/src/models.rs CardBatch. status stores the
-- SCREAMING_SNAKE_CASE variant, matching the serde renaming of BatchStatus
-- (DRAFT, SUBMITTED, IN_PRODUCTION, RECEIVED, DISTRIBUTED).

CREATE TABLE IF NOT EXISTS card_batches (
    id               UUID        NOT NULL PRIMARY KEY,
    election_id      UUID        NOT NULL,
    constituency_id  TEXT        NOT NULL,
    order_count      BIGINT      NOT NULL,
    manufacturer_ref TEXT,
    status           TEXT        NOT NULL
        CHECK (status IN ('DRAFT','SUBMITTED','IN_PRODUCTION','RECEIVED','DISTRIBUTED')),
    ordered_at       TIMESTAMPTZ NOT NULL,
    received_at      TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_card_batches_election     ON card_batches (election_id);
CREATE INDEX IF NOT EXISTS idx_card_batches_constituency ON card_batches (constituency_id);
