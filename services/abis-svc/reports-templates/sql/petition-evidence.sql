-- petition-evidence-bundle: election petition evidence per case.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    case_ref,
    subject_id_hash,     -- hashed; no PII in this view
    constituency_code,
    event_type,
    event_at,
    operator_id,
    tlog_leaf_index,
    tlog_root_hash
FROM eel_petition_evidence_view
WHERE event_at >= '{{ window_from }}'::timestamptz
  AND event_at <  '{{ window_to }}'::timestamptz
ORDER BY case_ref, event_at;
