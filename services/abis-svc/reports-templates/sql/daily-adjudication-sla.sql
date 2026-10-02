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

-- daily-adjudication-sla: SLA compliance per adjudication type.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    adjudication_type,
    COUNT(*)                            AS total_cases,
    COUNT(*) FILTER (WHERE completed_at - created_at <= INTERVAL '24 hours') AS within_sla,
    ROUND(
        100.0 * COUNT(*) FILTER (WHERE completed_at - created_at <= INTERVAL '24 hours')
        / NULLIF(COUNT(*), 0), 2
    )                                   AS sla_pct
FROM eel_adjudications
WHERE created_at >= '{{ window_from }}'::timestamptz
  AND created_at <  '{{ window_to }}'::timestamptz
  AND completed_at IS NOT NULL
GROUP BY adjudication_type
ORDER BY adjudication_type;
