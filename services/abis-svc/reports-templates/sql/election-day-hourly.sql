-- election-day-hourly: election-day hourly snapshot.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    DATE_TRUNC('hour', snapshot_at)     AS hour,
    county_code,
    polling_stations_active,
    total_enrolments_today,
    incidents_open,
    system_health_score
FROM eel_election_day_snapshots
WHERE snapshot_at >= '{{ window_from }}'::timestamptz
  AND snapshot_at <  '{{ window_to }}'::timestamptz
ORDER BY hour, county_code;
