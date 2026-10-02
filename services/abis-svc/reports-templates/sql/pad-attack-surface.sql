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

-- pad-attack-surface: presentation attack detection metrics.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    attack_type,
    modality,
    COUNT(*)                            AS detected_attacks,
    COUNT(*) FILTER (WHERE blocked)     AS blocked_count,
    ROUND(100.0 * COUNT(*) FILTER (WHERE blocked) / NULLIF(COUNT(*), 0), 2) AS block_rate_pct
FROM eel_pad_events
WHERE detected_at >= '{{ window_from }}'::timestamptz
  AND detected_at <  '{{ window_to }}'::timestamptz
GROUP BY attack_type, modality
ORDER BY attack_type, modality;
