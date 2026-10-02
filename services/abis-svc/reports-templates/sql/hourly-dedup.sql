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

-- hourly-dedup-queue-depth: deduplication queue depth sampled hourly.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    DATE_TRUNC('hour', sampled_at)  AS hour_bucket,
    queue_name,
    AVG(depth)                      AS avg_depth,
    MAX(depth)                      AS max_depth,
    MIN(depth)                      AS min_depth
FROM eel_dedup_queue_metrics
WHERE sampled_at >= '{{ window_from }}'::timestamptz
  AND sampled_at <  '{{ window_to }}'::timestamptz
GROUP BY DATE_TRUNC('hour', sampled_at), queue_name
ORDER BY hour_bucket, queue_name;
