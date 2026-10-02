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

-- ADR-0011 idempotency store: write operations are deduplicated by
-- (operation, Idempotency-Key) for 24 hours; repeat requests replay the
-- snapshotted success response instead of re-running side effects.
CREATE TABLE IF NOT EXISTS bvi_request_fingerprints (
  id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  operation       TEXT        NOT NULL,
  idempotency_key TEXT        NOT NULL,
  response_status INT         NOT NULL,
  response_body   JSONB,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at      TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '24 hours',
  CONSTRAINT bvi_request_fingerprints_op_key_uq UNIQUE (operation, idempotency_key)
);

CREATE INDEX IF NOT EXISTS bvi_request_fingerprints_expiry_idx
  ON bvi_request_fingerprints (expires_at);
