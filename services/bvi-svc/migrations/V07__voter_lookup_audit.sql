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

-- Per-lookup audit trail (M07 FR-M07-020).
-- Not erasable even on SAR (FR-M07-021).
CREATE TABLE IF NOT EXISTS voter_lookup_audit (
  id                   UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  actor_session_hash   TEXT        NOT NULL,
  target_voter_id_hash TEXT        NOT NULL,
  ts                   TIMESTAMPTZ NOT NULL DEFAULT now(),
  ip_hash              TEXT        NOT NULL,
  result_status        TEXT        NOT NULL CHECK (result_status IN ('found','not_found','rate_limited','error'))
);

CREATE INDEX ON voter_lookup_audit (ts DESC);
CREATE INDEX ON voter_lookup_audit (actor_session_hash);
