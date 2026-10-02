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

-- vendor-sla-credits-quarterly: vendor SLA credit calculations.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    vendor_id,
    contract_ref,
    sla_metric,
    agreed_threshold,
    actual_value,
    breach_count,
    credit_kes,
    period_end
FROM eel_vendor_sla_credits
WHERE period_end >= '{{ window_from }}'::timestamptz
  AND period_end <  '{{ window_to }}'::timestamptz
ORDER BY vendor_id, sla_metric;
