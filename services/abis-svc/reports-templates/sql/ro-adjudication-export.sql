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
