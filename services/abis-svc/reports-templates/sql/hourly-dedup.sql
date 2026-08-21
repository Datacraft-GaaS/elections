-- hourly-dedup-queue-depth: deduplication queue depth sampled hourly.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    DATE_TRUNC('hour', sampled_at)  AS hour_bucket,
    queue_name,
    AVG(depth)                      AS avg_depth,
    MAX(depth)                      AS max_depth,
    MIN(depth)                      AS min_depth
FROM eel_dedup_queue_metrics
WHERE sampled_at >= '{{ window_from }}'::timestamptz
  AND sampled_at <  '{{ window_to }}'::timestamptz
GROUP BY DATE_TRUNC('hour', sampled_at), queue_name
ORDER BY hour_bucket, queue_name;
