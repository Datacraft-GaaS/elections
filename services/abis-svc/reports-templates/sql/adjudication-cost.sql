-- adjudication-cost-monthly: adjudication cost per case.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    adjudication_type,
    COUNT(*)                            AS case_count,
    SUM(cost_kes)                       AS total_cost_kes,
    ROUND(AVG(cost_kes), 2)            AS avg_cost_kes,
    ROUND(PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY cost_kes), 2) AS median_cost_kes
FROM eel_adjudication_costs
WHERE period_start >= '{{ window_from }}'::timestamptz
  AND period_start <  '{{ window_to }}'::timestamptz
GROUP BY adjudication_type
ORDER BY adjudication_type;
