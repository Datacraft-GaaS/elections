-- ecc-daily-summary: ECC operational daily summary.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    station_id,
    county_code,
    enrolment_count,
    dedup_queue_depth,
    adjudication_sla_pct,
    vendor_health_pct,
    incidents_open,
    reported_at
FROM eel_ecc_daily_summary
WHERE reported_at >= '{{ window_from }}'::timestamptz
  AND reported_at <  '{{ window_to }}'::timestamptz
ORDER BY reported_at, county_code, station_id;
