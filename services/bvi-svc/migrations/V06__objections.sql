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

-- BVV objection workflow (M07).
CREATE TABLE IF NOT EXISTS objections (
  id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  obj_id                UUID        NOT NULL UNIQUE DEFAULT gen_random_uuid(),
  objector_session_hash TEXT        NOT NULL,
  target_voter_id       TEXT        NOT NULL,
  objection_type        TEXT        NOT NULL CHECK (objection_type IN ('rectification','transfer_request','removal_request')),
  justification         TEXT        NOT NULL,
  evidence_ref          TEXT,
  status                TEXT        NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','under_review','approved','rejected')),
  resolution            TEXT,
  filed_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
  resolved_at           TIMESTAMPTZ,
  resolved_by           TEXT
);

CREATE INDEX ON objections (status);
CREATE INDEX ON objections (target_voter_id);
CREATE INDEX ON objections (filed_at DESC);
