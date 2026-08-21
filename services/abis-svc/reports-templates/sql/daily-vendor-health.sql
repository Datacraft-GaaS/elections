-- daily-vendor-health: vendor SDK health metrics.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    vendor_id,
    sdk_version,
    COUNT(*)                            AS total_calls,
    COUNT(*) FILTER (WHERE success)     AS success_count,
    ROUND(AVG(latency_ms), 2)           AS avg_latency_ms,
    ROUND(PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms), 2) AS p99_latency_ms
FROM eel_vendor_calls
WHERE called_at >= '{{ window_from }}'::timestamptz
  AND called_at <  '{{ window_to }}'::timestamptz
GROUP BY vendor_id, sdk_version
ORDER BY vendor_id, sdk_version;
