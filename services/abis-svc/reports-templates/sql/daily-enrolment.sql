-- daily-enrolment-throughput: total enrolments per station per day.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    station_id,
    DATE(enrolled_at)        AS enrolment_date,
    COUNT(*)                 AS enrolment_count,
    COUNT(*) FILTER (WHERE status = 'complete')  AS complete_count,
    COUNT(*) FILTER (WHERE status = 'failed')    AS failed_count
FROM eel_enrolments
WHERE enrolled_at >= '{{ window_from }}'::timestamptz
  AND enrolled_at <  '{{ window_to }}'::timestamptz
GROUP BY station_id, DATE(enrolled_at)
ORDER BY enrolment_date, station_id;
