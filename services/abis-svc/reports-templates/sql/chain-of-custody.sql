-- chain-of-custody-per-subject: custody events per enrolment.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    subject_id_hash,
    event_type,
    event_at,
    operator_id,
    station_id,
    tlog_leaf_index
FROM eel_custody_events
WHERE event_at >= '{{ window_from }}'::timestamptz
  AND event_at <  '{{ window_to }}'::timestamptz
ORDER BY subject_id_hash, event_at;
