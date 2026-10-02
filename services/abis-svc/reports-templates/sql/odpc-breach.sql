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
