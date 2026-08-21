-- public-fmr-fnmr-monthly: public monthly FMR/FNMR summary (aggregated, no PII).
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    DATE_TRUNC('month', evaluated_at)   AS month,
    modality,
    ROUND(AVG(fmr) * 100, 4)           AS avg_fmr_pct,
    ROUND(AVG(fnmr) * 100, 4)          AS avg_fnmr_pct,
    COUNT(*)                            AS sample_count
FROM eel_biometric_accuracy_metrics
WHERE evaluated_at >= '{{ window_from }}'::timestamptz
  AND evaluated_at <  '{{ window_to }}'::timestamptz
GROUP BY DATE_TRUNC('month', evaluated_at), modality
ORDER BY month, modality;
