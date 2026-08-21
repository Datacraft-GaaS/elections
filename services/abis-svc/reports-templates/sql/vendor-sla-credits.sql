-- vendor-sla-credits-quarterly: vendor SLA credit calculations.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    vendor_id,
    contract_ref,
    sla_metric,
    agreed_threshold,
    actual_value,
    breach_count,
    credit_kes,
    period_end
FROM eel_vendor_sla_credits
WHERE period_end >= '{{ window_from }}'::timestamptz
  AND period_end <  '{{ window_to }}'::timestamptz
ORDER BY vendor_id, sla_metric;
