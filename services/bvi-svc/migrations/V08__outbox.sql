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

-- Transactional outbox (ADR-0015). Debezium reads this table via logical replication.
CREATE TABLE IF NOT EXISTS bvi_outbox (
  id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id      UUID        NOT NULL UNIQUE DEFAULT gen_random_uuid(),
  topic         TEXT        NOT NULL,
  partition_key TEXT,
  payload       JSONB       NOT NULL,
  headers       JSONB       NOT NULL DEFAULT '{}',
  signature     TEXT,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  published_at  TIMESTAMPTZ
);

CREATE INDEX ON bvi_outbox (created_at) WHERE published_at IS NULL;
