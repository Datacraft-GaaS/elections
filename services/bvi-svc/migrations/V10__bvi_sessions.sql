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

-- BVI device session lifecycle (openBviSession / closeBviSession).
-- A device may hold at most one active session per election day; the partial
-- unique index turns a second open into a conflict the API maps to 409.
CREATE TABLE IF NOT EXISTS bvi_sessions (
  id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  session_id    TEXT        NOT NULL UNIQUE,
  station_code  TEXT        NOT NULL,
  device_serial TEXT        NOT NULL,
  election_id   TEXT        NOT NULL,
  status        TEXT        NOT NULL DEFAULT 'active'
                CHECK (status IN ('active','closed','revoked')),
  opened_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  closed_at     TIMESTAMPTZ
);

CREATE UNIQUE INDEX IF NOT EXISTS bvi_sessions_device_election_active_uq
  ON bvi_sessions (device_serial, election_id)
  WHERE status = 'active';

CREATE INDEX IF NOT EXISTS bvi_sessions_station_idx ON bvi_sessions (station_code);

-- Link identification events back to their originating session so the
-- identifications_count reported on BviSession is a real count.
ALTER TABLE identification_events ADD COLUMN IF NOT EXISTS session_id TEXT;
CREATE INDEX IF NOT EXISTS identification_events_session_idx
  ON identification_events (session_id);

-- The contract exposes decisions (fallback_used, ambiguous) that the original
-- outcome CHECK did not allow; widen instead of lossy remapping to 'error'.
ALTER TABLE identification_events DROP CONSTRAINT IF EXISTS identification_events_outcome_check;
ALTER TABLE identification_events ADD CONSTRAINT identification_events_outcome_check
  CHECK (outcome IN ('matched','no_match','timeout','error','fallback_used','ambiguous'));
