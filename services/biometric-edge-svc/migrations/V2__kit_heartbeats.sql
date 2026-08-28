-- Migration V2: biometric_kit_heartbeats hot table (WP-3.8)
-- Short-retention telemetry. Partitioned by day; retention 30 days.
CREATE TABLE biometric_kit_heartbeats (
    id               UUID NOT NULL DEFAULT gen_random_uuid(),
    device_serial    TEXT NOT NULL,
    station_code     TEXT NOT NULL,
    recorded_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    battery_pct      SMALLINT,
    gps_lat          DOUBLE PRECISION,
    gps_lon          DOUBLE PRECISION,
    gnss_fix_quality SMALLINT,
    network_rssi_dbm SMALLINT,
    uptime_s         BIGINT,
    enrolled_count   INTEGER,
    PRIMARY KEY (id, recorded_at)
) PARTITION BY RANGE (recorded_at);

-- Seed current-month partition (appset-gen creates rolling partitions via cron).
CREATE TABLE biometric_kit_heartbeats_default PARTITION OF biometric_kit_heartbeats DEFAULT;

CREATE INDEX biometric_kit_heartbeats_device_recorded
    ON biometric_kit_heartbeats (device_serial, recorded_at DESC);

COMMENT ON TABLE biometric_kit_heartbeats IS 'Kit telemetry heartbeats (30-day rolling retention, WP-3.8).';
