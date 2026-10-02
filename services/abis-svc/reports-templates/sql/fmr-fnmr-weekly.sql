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

-- fmr-fnmr-weekly: false match rate / false non-match rate trend.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    DATE_TRUNC('day', evaluated_at)     AS day,
    modality,
    ROUND(AVG(fmr) * 100, 4)           AS avg_fmr_pct,
    ROUND(AVG(fnmr) * 100, 4)          AS avg_fnmr_pct,
    COUNT(*)                            AS evaluation_count
FROM eel_biometric_accuracy_metrics
WHERE evaluated_at >= '{{ window_from }}'::timestamptz
  AND evaluated_at <  '{{ window_to }}'::timestamptz
GROUP BY DATE_TRUNC('day', evaluated_at), modality
ORDER BY day, modality;
