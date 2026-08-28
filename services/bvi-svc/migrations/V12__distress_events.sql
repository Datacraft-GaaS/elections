-- Distress / duress signalling (raiseDistress). Rows are persisted even if
-- the NOC notification pipeline is unavailable; NOC delivery happens via the
-- transactional outbox event eel.bvi.distress_raised.v1.
CREATE TABLE IF NOT EXISTS distress_events (
  id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  station_code   TEXT        NOT NULL,
  device_serial  TEXT        NOT NULL,
  distress_type  TEXT        NOT NULL CHECK (distress_type IN
                ('officer_duress','device_tamper','power_failure',
                 'network_failure','physical_intrusion','other')),
  details        TEXT,
  raised_at      TIMESTAMPTZ NOT NULL,
  received_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS distress_events_raised_idx ON distress_events (raised_at DESC);
CREATE INDEX IF NOT EXISTS distress_events_station_idx ON distress_events (station_code);
