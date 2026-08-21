-- iso-27001-annual: ISO 27001 control evidence.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    control_id,
    control_name,
    control_category,
    implementation_status,
    last_audit_at,
    auditor_ref,
    finding_count,
    open_ncr_count
FROM eel_iso27001_controls
WHERE last_audit_at >= '{{ window_from }}'::timestamptz
  AND last_audit_at <  '{{ window_to }}'::timestamptz
ORDER BY control_id;
