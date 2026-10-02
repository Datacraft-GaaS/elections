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

-- Section 44A dual-officer verification confirmations per polling station.
-- Officer 1 (presiding officer) opens the confirmation; officer 2 (deputy)
-- completes it. One confirmation row per station; both officers must differ.

CREATE TABLE IF NOT EXISTS s44a_confirmations (
    confirmation_id       UUID PRIMARY KEY,
    county_code           TEXT        NOT NULL,
    station_code          TEXT        NOT NULL,
    officer1_id           TEXT        NOT NULL,
    officer1_confirmed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    officer2_id           TEXT        NULL,
    officer2_confirmed_at TIMESTAMPTZ NULL,
    status                TEXT        NOT NULL DEFAULT 'officer1_confirmed'
                                      CHECK (status IN ('officer1_confirmed', 'complete')),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (station_code)
);

CREATE INDEX IF NOT EXISTS idx_s44a_confirmations_county ON s44a_confirmations (county_code, status);
