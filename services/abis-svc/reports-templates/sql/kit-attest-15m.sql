-- kit-attestation-15m: kit attestation status snapshot.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    kit_id,
    last_attested_at,
    attestation_status,
    firmware_version,
    tpm_ek_hash
FROM eel_kit_attestations
WHERE last_attested_at >= '{{ window_from }}'::timestamptz
  AND last_attested_at <  '{{ window_to }}'::timestamptz
ORDER BY last_attested_at DESC;
