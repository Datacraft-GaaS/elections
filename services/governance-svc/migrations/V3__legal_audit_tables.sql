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

-- M62 Transmission Legal Basis Audit tables (WP-5.1)

CREATE TABLE legal_transmission_records (
    record_id           TEXT PRIMARY KEY,
    event_id            TEXT NOT NULL,
    event_type          TEXT NOT NULL,
    transmission_step   TEXT NOT NULL,
    station_code        TEXT,
    officer_id_hash     TEXT,
    legal_basis_refs    JSONB NOT NULL DEFAULT '[]',
    occurred_at         TIMESTAMPTZ NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE legal_compliance_verdicts (
    verdict_id          TEXT PRIMARY KEY,
    record_id           TEXT NOT NULL REFERENCES legal_transmission_records(record_id),
    rule_id             TEXT NOT NULL,
    rule_section        TEXT NOT NULL,
    passed              BOOLEAN NOT NULL,
    failure_reason      TEXT,
    tlog_hash           TEXT NOT NULL,
    verified_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE legal_rules (
    rule_id             TEXT PRIMARY KEY,
    section_ref         TEXT NOT NULL,
    description         TEXT NOT NULL,
    applies_to          TEXT NOT NULL,
    active              BOOLEAN NOT NULL DEFAULT TRUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_verdicts_record ON legal_compliance_verdicts(record_id);
CREATE INDEX idx_verdicts_passed ON legal_compliance_verdicts(passed);
CREATE INDEX idx_transmissions_type ON legal_transmission_records(event_type);
