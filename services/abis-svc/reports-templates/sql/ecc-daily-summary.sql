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

-- ecc-daily-summary: ECC operational daily summary.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    station_id,
    county_code,
    enrolment_count,
    dedup_queue_depth,
    adjudication_sla_pct,
    vendor_health_pct,
    incidents_open,
    reported_at
FROM eel_ecc_daily_summary
WHERE reported_at >= '{{ window_from }}'::timestamptz
  AND reported_at <  '{{ window_to }}'::timestamptz
ORDER BY reported_at, county_code, station_id;
