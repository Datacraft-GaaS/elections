-- Turnout rollup base table (materialised views refresh incrementally).
CREATE TABLE IF NOT EXISTS turnout_rollups (
  id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  scope               TEXT        NOT NULL CHECK (scope IN ('station','constituency','county','national')),
  scope_code          TEXT        NOT NULL,
  authenticated_count BIGINT      NOT NULL DEFAULT 0,
  registered_count    BIGINT      NOT NULL DEFAULT 0,
  updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT turnout_rollups_scope_code_uq UNIQUE (scope, scope_code)
);

CREATE INDEX ON turnout_rollups (scope, updated_at DESC);

-- Materialised view: per-station.
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_turnout_station AS
  SELECT station_code AS scope_code, COUNT(*) AS authenticated_count
  FROM identification_events
  WHERE outcome = 'matched'
  GROUP BY station_code
WITH DATA;

CREATE UNIQUE INDEX IF NOT EXISTS mv_turnout_station_uq ON mv_turnout_station (scope_code);
