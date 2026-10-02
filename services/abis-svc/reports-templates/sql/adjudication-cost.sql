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

-- adjudication-cost-monthly: adjudication cost per case.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    adjudication_type,
    COUNT(*)                            AS case_count,
    SUM(cost_kes)                       AS total_cost_kes,
    ROUND(AVG(cost_kes), 2)            AS avg_cost_kes,
    ROUND(PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY cost_kes), 2) AS median_cost_kes
FROM eel_adjudication_costs
WHERE period_start >= '{{ window_from }}'::timestamptz
  AND period_start <  '{{ window_to }}'::timestamptz
GROUP BY adjudication_type
ORDER BY adjudication_type;
