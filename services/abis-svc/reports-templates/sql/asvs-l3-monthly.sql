-- asvs-l3-monthly: ASVS L3 security evidence.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    control_id,
    control_description,
    status,
    evidence_ref,
    last_verified_at,
    verified_by
FROM eel_asvs_evidence
WHERE last_verified_at >= '{{ window_from }}'::timestamptz
  AND last_verified_at <  '{{ window_to }}'::timestamptz
ORDER BY control_id;
