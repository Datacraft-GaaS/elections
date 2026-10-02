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

-- model-drift-weekly: model performance drift alerts.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    model_id,
    model_version,
    drift_metric,
    baseline_value,
    current_value,
    ABS(current_value - baseline_value) AS abs_drift,
    alerted_at
FROM eel_model_drift_alerts
WHERE alerted_at >= '{{ window_from }}'::timestamptz
  AND alerted_at <  '{{ window_to }}'::timestamptz
ORDER BY alerted_at DESC, model_id;
