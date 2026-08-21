-- daily-shard-load: biometric matching shard load distribution.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    shard_id,
    COUNT(*)                            AS match_requests,
    ROUND(AVG(cpu_pct), 2)             AS avg_cpu_pct,
    ROUND(AVG(memory_mb), 2)           AS avg_memory_mb,
    MAX(cpu_pct)                        AS peak_cpu_pct
FROM eel_shard_metrics
WHERE recorded_at >= '{{ window_from }}'::timestamptz
  AND recorded_at <  '{{ window_to }}'::timestamptz
GROUP BY shard_id
ORDER BY shard_id;
