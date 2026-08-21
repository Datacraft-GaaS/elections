-- iab-oversight-quarterly: IAB oversight card.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    metric_name,
    metric_value,
    target_value,
    status,
    notes,
    reported_at
FROM eel_iab_oversight_metrics
WHERE reported_at >= '{{ window_from }}'::timestamptz
  AND reported_at <  '{{ window_to }}'::timestamptz
ORDER BY metric_name, reported_at;
