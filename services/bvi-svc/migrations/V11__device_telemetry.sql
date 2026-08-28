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
