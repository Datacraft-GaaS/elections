-- odpc-breach-24h: data breach notification evidence.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    incident_id,
    incident_type,
    detected_at,
    notified_odpc_at,
    affected_data_categories,
    estimated_subjects_affected,
    containment_status,
    dpo_sign_off
FROM eel_data_breach_incidents
WHERE detected_at >= '{{ window_from }}'::timestamptz
  AND detected_at <  '{{ window_to }}'::timestamptz
ORDER BY detected_at DESC;
