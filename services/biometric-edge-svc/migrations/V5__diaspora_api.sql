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

-- Diaspora API surface — postgres for public contract (no in-memory)
-- Replaces Mutex<HashMap<String, Value>> embassies/ballot_dispatches/return_dispatches in diaspora/api.rs

CREATE TABLE IF NOT EXISTS diaspora_api_embassies (
    embassy_code    TEXT PRIMARY KEY,
    data            JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS diaspora_api_ballot_dispatches (
    dispatch_id     TEXT PRIMARY KEY,
    data            JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS diaspora_api_return_dispatches (
    return_id       TEXT PRIMARY KEY,
    data            JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_diaspora_embassies_code ON diaspora_api_embassies(embassy_code);
CREATE INDEX IF NOT EXISTS idx_diaspora_ballot_dispatches_id ON diaspora_api_ballot_dispatches(dispatch_id);

-- Reversible: DROP TABLE IF EXISTS diaspora_api_return_dispatches, diaspora_api_ballot_dispatches, diaspora_api_embassies CASCADE;
