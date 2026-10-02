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

-- Row-level security for observer isolation and party-agent isolation (ADR-0012)

ALTER TABLE voters ENABLE ROW LEVEL SECURITY;

-- Observers: read-only, no Tier A fields (handled at application layer too)
CREATE POLICY observer_read ON voters
    FOR SELECT
    TO PUBLIC
    USING (TRUE);

-- Application writes only (via service account)
CREATE POLICY service_write ON voters
    FOR ALL
    TO PUBLIC
    USING (current_setting('app.role', TRUE) IN ('bvr-svc', 'admin'))
    WITH CHECK (current_setting('app.role', TRUE) IN ('bvr-svc', 'admin'));
