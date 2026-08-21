-- quality-distribution-monthly: biometric template quality distribution.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    modality,
    quality_band,
    COUNT(*)                            AS count,
    ROUND(100.0 * COUNT(*) / SUM(COUNT(*)) OVER (PARTITION BY modality), 2) AS pct
FROM eel_template_quality_samples
WHERE sampled_at >= '{{ window_from }}'::timestamptz
  AND sampled_at <  '{{ window_to }}'::timestamptz
GROUP BY modality, quality_band
ORDER BY modality, quality_band;
