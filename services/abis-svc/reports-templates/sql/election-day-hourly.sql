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

-- election-day-hourly: election-day hourly snapshot.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    DATE_TRUNC('hour', snapshot_at)     AS hour,
    county_code,
    polling_stations_active,
    total_enrolments_today,
    incidents_open,
    system_health_score
FROM eel_election_day_snapshots
WHERE snapshot_at >= '{{ window_from }}'::timestamptz
  AND snapshot_at <  '{{ window_to }}'::timestamptz
ORDER BY hour, county_code;
