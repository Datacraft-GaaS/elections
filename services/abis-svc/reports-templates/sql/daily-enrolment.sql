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

-- daily-enrolment-throughput: total enrolments per station per day.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    station_id,
    DATE(enrolled_at)        AS enrolment_date,
    COUNT(*)                 AS enrolment_count,
    COUNT(*) FILTER (WHERE status = 'complete')  AS complete_count,
    COUNT(*) FILTER (WHERE status = 'failed')    AS failed_count
FROM eel_enrolments
WHERE enrolled_at >= '{{ window_from }}'::timestamptz
  AND enrolled_at <  '{{ window_to }}'::timestamptz
GROUP BY station_id, DATE(enrolled_at)
ORDER BY enrolment_date, station_id;
