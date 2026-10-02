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
