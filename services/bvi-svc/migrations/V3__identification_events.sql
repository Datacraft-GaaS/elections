-- Identification event mirror (Kafka is source of truth; PG for reads).
-- EC-DAT-01: unique on event_id for idempotency.
CREATE TABLE IF NOT EXISTS identification_events (
  id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id        UUID        NOT NULL UNIQUE,
  voter_id        TEXT        NOT NULL,
  station_code    TEXT        NOT NULL,
  kit_serial      TEXT        NOT NULL,
  modality        TEXT        NOT NULL CHECK (modality IN ('fingerprint','iris','face','nfc')),
  outcome         TEXT        NOT NULL CHECK (outcome IN ('matched','no_match','timeout','error')),
  ts_hlc          TEXT        NOT NULL,
  gps_fix         JSONB,
  kit_sig         TEXT,
  received_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  honey_mode      BOOLEAN     NOT NULL DEFAULT false
);

CREATE INDEX ON identification_events (station_code);
CREATE INDEX ON identification_events (kit_serial);
CREATE INDEX ON identification_events (received_at DESC);
-- EC-OPS-02: no-double-ID per election; voter_id + election_id unique (kit_id updated on legitimate swap).
-- election_id added when multi-election support is needed; placeholder column included now.
ALTER TABLE identification_events ADD COLUMN IF NOT EXISTS election_id TEXT NOT NULL DEFAULT 'EL-2027';
CREATE UNIQUE INDEX IF NOT EXISTS identification_events_voter_election_uq
  ON identification_events (voter_id, election_id)
  WHERE outcome = 'matched';
