-- dpia-quarterly: DPIA evidence pack query.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    dpia_id,
    data_category,
    processing_purpose,
    legal_basis,
    retention_days,
    risk_level,
    mitigation_applied,
    reviewed_at
FROM eel_dpia_records
WHERE reviewed_at >= '{{ window_from }}'::timestamptz
  AND reviewed_at <  '{{ window_to }}'::timestamptz
ORDER BY reviewed_at DESC;
