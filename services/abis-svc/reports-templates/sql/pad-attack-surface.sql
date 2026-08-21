-- pad-attack-surface: presentation attack detection metrics.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    attack_type,
    modality,
    COUNT(*)                            AS detected_attacks,
    COUNT(*) FILTER (WHERE blocked)     AS blocked_count,
    ROUND(100.0 * COUNT(*) FILTER (WHERE blocked) / NULLIF(COUNT(*), 0), 2) AS block_rate_pct
FROM eel_pad_events
WHERE detected_at >= '{{ window_from }}'::timestamptz
  AND detected_at <  '{{ window_to }}'::timestamptz
GROUP BY attack_type, modality
ORDER BY attack_type, modality;
