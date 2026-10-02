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

-- daily-shard-load: biometric matching shard load distribution.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    shard_id,
    COUNT(*)                            AS match_requests,
    ROUND(AVG(cpu_pct), 2)             AS avg_cpu_pct,
    ROUND(AVG(memory_mb), 2)           AS avg_memory_mb,
    MAX(cpu_pct)                        AS peak_cpu_pct
FROM eel_shard_metrics
WHERE recorded_at >= '{{ window_from }}'::timestamptz
  AND recorded_at <  '{{ window_to }}'::timestamptz
GROUP BY shard_id
ORDER BY shard_id;
