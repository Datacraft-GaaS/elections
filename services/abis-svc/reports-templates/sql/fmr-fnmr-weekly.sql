-- fmr-fnmr-weekly: false match rate / false non-match rate trend.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    DATE_TRUNC('day', evaluated_at)     AS day,
    modality,
    ROUND(AVG(fmr) * 100, 4)           AS avg_fmr_pct,
    ROUND(AVG(fnmr) * 100, 4)          AS avg_fnmr_pct,
    COUNT(*)                            AS evaluation_count
FROM eel_biometric_accuracy_metrics
WHERE evaluated_at >= '{{ window_from }}'::timestamptz
  AND evaluated_at <  '{{ window_to }}'::timestamptz
GROUP BY DATE_TRUNC('day', evaluated_at), modality
ORDER BY day, modality;
