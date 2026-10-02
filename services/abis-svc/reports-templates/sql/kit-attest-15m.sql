-- Copyright (c) 2026 Nyimbi Odero
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.

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
