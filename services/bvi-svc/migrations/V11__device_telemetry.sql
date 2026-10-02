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

-- Device health telemetry (M05/M06, submitTelemetry). Data-class D:
-- no voter or officer identifiers, safe to aggregate.
CREATE TABLE IF NOT EXISTS device_telemetry (
  id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  device_serial       TEXT        NOT NULL,
  station_code        TEXT        NOT NULL,
  battery_pct         INT         NOT NULL CHECK (battery_pct BETWEEN 0 AND 100),
  signal_strength_dbm DOUBLE PRECISION,
  temperature_c       DOUBLE PRECISION,
  ts                  TIMESTAMPTZ NOT NULL,
  received_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS device_telemetry_serial_ts_idx
  ON device_telemetry (device_serial, ts DESC);
CREATE INDEX IF NOT EXISTS device_telemetry_station_idx ON device_telemetry (station_code);
