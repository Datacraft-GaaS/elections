-- fairness-quarterly: demographic fairness analysis (aggregated, no PII).
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    demographic_group,
    modality,
    ROUND(AVG(fmr) * 100, 4)           AS avg_fmr_pct,
    ROUND(AVG(fnmr) * 100, 4)          AS avg_fnmr_pct,
    COUNT(*)                            AS sample_size,
    ROUND(STDDEV(fmr) * 100, 6)        AS fmr_stddev
FROM eel_fairness_evaluations
WHERE evaluated_at >= '{{ window_from }}'::timestamptz
  AND evaluated_at <  '{{ window_to }}'::timestamptz
GROUP BY demographic_group, modality
ORDER BY demographic_group, modality;
