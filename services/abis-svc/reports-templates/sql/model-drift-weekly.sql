-- model-drift-weekly: model performance drift alerts.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    model_id,
    model_version,
    drift_metric,
    baseline_value,
    current_value,
    ABS(current_value - baseline_value) AS abs_drift,
    alerted_at
FROM eel_model_drift_alerts
WHERE alerted_at >= '{{ window_from }}'::timestamptz
  AND alerted_at <  '{{ window_to }}'::timestamptz
ORDER BY alerted_at DESC, model_id;
