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

-- Audit shadow table: mirrors sensitive operations for T1-audit per M03.
-- Data class D (metadata, no voter PII) per ADR-0016.
CREATE TABLE IF NOT EXISTS eel_abis_audit_shadow (
    audit_id         TEXT        PRIMARY KEY,
    operation        TEXT        NOT NULL,  -- 'match_1n','match_11','adjudication_open','adjudication_resolve','batch_dedup'
    actor_id         TEXT,
    resource_id      TEXT,
    result_code      TEXT,
    client_id        TEXT,
    request_id       TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_eel_abis_audit_created ON eel_abis_audit_shadow(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_eel_abis_audit_op      ON eel_abis_audit_shadow(operation);
