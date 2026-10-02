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

-- M21 RLA tables (WP-5.1)

CREATE TABLE rla_audit_runs (
    run_id          TEXT PRIMARY KEY,
    election_id     TEXT NOT NULL,
    position_code   TEXT NOT NULL,
    audit_method    TEXT NOT NULL CHECK (audit_method IN ('ballot_polling','comparison','batch_comparison')),
    risk_limit      REAL NOT NULL CHECK (risk_limit > 0 AND risk_limit < 1),
    seed_ref        TEXT,
    status          TEXT NOT NULL DEFAULT 'initialised'
                        CHECK (status IN ('initialised','sampling','counting','complete','escalated')),
    completed_at    TIMESTAMPTZ,
    sig             TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE rla_entropy_sources (
    source_id       TEXT PRIMARY KEY,
    run_id          TEXT NOT NULL REFERENCES rla_audit_runs(run_id),
    source_type     TEXT NOT NULL CHECK (source_type IN ('dice','beacon','hardware','iab_card')),
    value_hash      TEXT NOT NULL,
    witnessed_at    TIMESTAMPTZ NOT NULL,
    witness_sigs    JSONB NOT NULL DEFAULT '[]'
);

CREATE TABLE rla_sampled_ballots (
    ballot_id               TEXT PRIMARY KEY,
    run_id                  TEXT NOT NULL REFERENCES rla_audit_runs(run_id),
    station_code            TEXT NOT NULL,
    batch_id                TEXT NOT NULL,
    ballot_position         INTEGER NOT NULL,
    auditor_interpretation  TEXT,
    discrepancy             BOOLEAN NOT NULL DEFAULT FALSE,
    verdict                 TEXT,
    recorded_at             TIMESTAMPTZ,
    synced                  BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE rla_audit_reports (
    report_id           TEXT PRIMARY KEY,
    run_id              TEXT NOT NULL REFERENCES rla_audit_runs(run_id),
    conclusion          TEXT NOT NULL CHECK (conclusion IN ('pass','escalate','recount_triggered')),
    risk_achieved       REAL,
    discrepancy_count   INTEGER NOT NULL DEFAULT 0,
    sig                 TEXT,
    published_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE rla_audit_manifests (
    manifest_id     TEXT PRIMARY KEY,
    run_id          TEXT NOT NULL REFERENCES rla_audit_runs(run_id),
    station_count   INTEGER NOT NULL,
    total_ballots   INTEGER NOT NULL,
    manifest_hash   TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE rla_witness_records (
    witness_id      TEXT PRIMARY KEY,
    run_id          TEXT NOT NULL REFERENCES rla_audit_runs(run_id),
    name            TEXT NOT NULL,
    organisation    TEXT NOT NULL,
    signature_b64   TEXT,
    witnessed_at    TIMESTAMPTZ
);

CREATE INDEX idx_rla_ballots_run ON rla_sampled_ballots(run_id);
CREATE INDEX idx_rla_entropy_run ON rla_entropy_sources(run_id);
