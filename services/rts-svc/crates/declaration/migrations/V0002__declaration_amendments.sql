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

-- Declaration amendment audit table (WP-3.3, M51, EC-LEG-11).
-- Retained 7 years; no soft-delete path (AC-M51-17).

CREATE TABLE IF NOT EXISTS declaration_amendments (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    declaration_id          UUID        NOT NULL REFERENCES declarations(id),
    reason                  TEXT        NOT NULL,
    amended_by              TEXT        NOT NULL,
    approved_by             TEXT        NOT NULL,
    amended_at              TIMESTAMPTZ NOT NULL,
    previous_snapshot_hash  TEXT        NOT NULL,
    new_snapshot_hash        TEXT        NOT NULL,
    court_order_ref         TEXT,
    new_tally_event_id      TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Audit shadow rows: one per state-transition event, immutable.
CREATE TABLE IF NOT EXISTS declaration_audit_shadow (
    id             BIGSERIAL   PRIMARY KEY,
    declaration_id UUID        NOT NULL,
    event_type     TEXT        NOT NULL,
    actor_id       TEXT        NOT NULL,
    hlc_ts         TIMESTAMPTZ NOT NULL,
    document_hash  TEXT,
    payload_hash   TEXT        NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_decl_audit_decl_id
    ON declaration_audit_shadow (declaration_id, hlc_ts);
