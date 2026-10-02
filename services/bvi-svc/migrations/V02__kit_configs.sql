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

-- Per-station kit configuration bundles.
CREATE TABLE IF NOT EXISTS kit_configs (
  id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  station_code    TEXT        NOT NULL,
  version         INT         NOT NULL,
  bundle_id       UUID        NOT NULL UNIQUE,
  voter_list_hash TEXT        NOT NULL,
  geofence_hash   TEXT        NOT NULL,
  cert_hash       TEXT        NOT NULL,
  minio_key       TEXT        NOT NULL,
  signed_at       TIMESTAMPTZ NOT NULL,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT kit_configs_station_version_uq UNIQUE (station_code, version)
);

CREATE INDEX ON kit_configs (station_code);
CREATE INDEX ON kit_configs (created_at DESC);
