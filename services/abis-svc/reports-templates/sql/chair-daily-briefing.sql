-- chair-daily-briefing: executive daily briefing metrics.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    metric_category,
    metric_name,
    metric_value,
    delta_vs_yesterday,
    status_flag,
    reported_at
FROM eel_executive_briefing_metrics
WHERE reported_at >= '{{ window_from }}'::timestamptz
  AND reported_at <  '{{ window_to }}'::timestamptz
ORDER BY metric_category, metric_name;
