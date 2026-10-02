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

-- public-enrolment-totals: public weekly enrolment totals.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    DATE_TRUNC('week', enrolled_at)     AS week_start,
    county_code,
    COUNT(*)                            AS enrolments
FROM eel_enrolments
WHERE enrolled_at >= '{{ window_from }}'::timestamptz
  AND enrolled_at <  '{{ window_to }}'::timestamptz
  AND status = 'complete'
GROUP BY DATE_TRUNC('week', enrolled_at), county_code
ORDER BY week_start, county_code;
