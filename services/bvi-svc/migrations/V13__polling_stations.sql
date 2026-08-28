-- Polling station registry mirror used by getVoterStation. Populated by the
-- boundary sync job from boundary-svc boundaries (kind = 'polling_station');
-- this service never invents station metadata on its own.
CREATE TABLE IF NOT EXISTS polling_stations (
  station_code      TEXT PRIMARY KEY,
  name              TEXT NOT NULL,
  constituency_code TEXT NOT NULL,
  county_code       TEXT NOT NULL
);
