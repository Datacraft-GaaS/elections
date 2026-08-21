-- ro-adjudication-export: returning-officer adjudication export.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    adjudication_id,
    constituency_code,
    outcome,
    adjudicated_by,
    adjudicated_at,
    evidence_bundle_ref,
    appeal_deadline
FROM eel_ro_adjudications
WHERE adjudicated_at >= '{{ window_from }}'::timestamptz
  AND adjudicated_at <  '{{ window_to }}'::timestamptz
ORDER BY constituency_code, adjudicated_at;
