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

-- Register snapshots and Merkle roots
CREATE TABLE IF NOT EXISTS register_snapshots (
    snapshot_id     TEXT        NOT NULL PRIMARY KEY,
    election_scope  TEXT        NOT NULL,
    station_count   INTEGER     NOT NULL,
    voter_count     BIGINT      NOT NULL,
    merkle_root     TEXT        NOT NULL,
    signed_by       TEXT        NOT NULL,
    checkpoint      TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS merkle_roots (
    id              TEXT        NOT NULL PRIMARY KEY,
    root_hash       TEXT        NOT NULL,
    seq             BIGINT      NOT NULL,
    election_scope  TEXT        NOT NULL,
    signed_by_hsm   TEXT,
    escrow_confirmed BOOLEAN    NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_merkle_seq ON merkle_roots (election_scope, seq);
