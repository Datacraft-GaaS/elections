-- daily-adjudication-sla: SLA compliance per adjudication type.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    adjudication_type,
    COUNT(*)                            AS total_cases,
    COUNT(*) FILTER (WHERE completed_at - created_at <= INTERVAL '24 hours') AS within_sla,
    ROUND(
        100.0 * COUNT(*) FILTER (WHERE completed_at - created_at <= INTERVAL '24 hours')
        / NULLIF(COUNT(*), 0), 2
    )                                   AS sla_pct
FROM eel_adjudications
WHERE created_at >= '{{ window_from }}'::timestamptz
  AND created_at <  '{{ window_to }}'::timestamptz
  AND completed_at IS NOT NULL
GROUP BY adjudication_type
ORDER BY adjudication_type;
