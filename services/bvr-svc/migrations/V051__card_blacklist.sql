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

-- Card blacklist for lost/stolen voter ID cards (M53, WP-4.10).
-- Propagation to BVI edge nodes happens via the transactional outbox
-- (topic eel.bvr.card_blacklisted.v1); the row itself is the source of truth.

CREATE TABLE IF NOT EXISTS card_blacklist (
    id             UUID        NOT NULL PRIMARY KEY,
    card_number    TEXT        NOT NULL UNIQUE,
    reason         TEXT        NOT NULL,
    blacklisted_at TIMESTAMPTZ NOT NULL,
    source         TEXT        NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_card_blacklist_number ON card_blacklist (card_number);
