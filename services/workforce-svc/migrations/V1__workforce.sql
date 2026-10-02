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

-- workforce-svc schema (design: docs/workforce-lifecycle.md).
-- Five bounded contexts in one service: registry & contacts (S4), recruitment
-- pipeline (S1), postings & equipment claims (S5), tasking (S6), performance (S7).
-- Content-bearing identifiers are SHA-256 hex hashes; no raw PII beyond
-- appointment names (DPA 2019).

-- ---------------------------------------------------------------------------
-- S4: Worker registry & contacts
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS workforce_workers (
    worker_id             UUID PRIMARY KEY,
    legal_name            TEXT        NOT NULL,
    national_id_hash      CHAR(64)    NOT NULL UNIQUE,
    home_county           TEXT        NOT NULL,
    role_eligibility      JSONB       NOT NULL DEFAULT '[]',
    payroll_contract_ref  TEXT,
    next_of_kin_name      TEXT,
    next_of_kin_phone_hash CHAR(64),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workforce_workers_county ON workforce_workers (home_county);

CREATE TABLE IF NOT EXISTS workforce_contacts (
    contact_id     UUID PRIMARY KEY,
    worker_id      UUID        NOT NULL REFERENCES workforce_workers (worker_id),
    channel        TEXT        NOT NULL CHECK (channel IN ('phone', 'whatsapp', 'email', 'sms')),
    value_hash_hex CHAR(64)    NOT NULL,
    verified       BOOLEAN     NOT NULL DEFAULT FALSE,
    consented      BOOLEAN     NOT NULL DEFAULT FALSE,
    preferred      BOOLEAN     NOT NULL DEFAULT FALSE,
    changed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (worker_id, channel, value_hash_hex)
);

-- At most one preferred contact point per channel per worker.
CREATE UNIQUE INDEX IF NOT EXISTS idx_workforce_contacts_one_preferred
    ON workforce_contacts (worker_id, channel) WHERE preferred;

CREATE INDEX IF NOT EXISTS idx_workforce_contacts_worker ON workforce_contacts (worker_id);

-- Append-only change log.
CREATE TABLE IF NOT EXISTS workforce_contact_log (
    log_id   UUID PRIMARY KEY,
    worker_id UUID        NOT NULL REFERENCES workforce_workers (worker_id),
    channel  TEXT        NOT NULL,
    at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actor    TEXT        NOT NULL,
    action   TEXT        NOT NULL CHECK (action IN ('added', 'updated', 'removed', 'verified', 'consent_granted')),
    reason   TEXT
);

CREATE INDEX IF NOT EXISTS idx_workforce_contact_log_worker ON workforce_contact_log (worker_id, at);

-- ---------------------------------------------------------------------------
-- S1: Recruitment state machine
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS workforce_recruitment_cases (
    case_id          UUID PRIMARY KEY,
    application_ref  TEXT     NOT NULL UNIQUE,
    applicant_name   TEXT     NOT NULL,
    national_id_hash CHAR(64) NOT NULL,
    role_code        TEXT     NOT NULL,
    county           TEXT     NOT NULL,
    stage            TEXT     NOT NULL DEFAULT 'applied'
                     CHECK (stage IN ('applied', 'screened', 'documents_complete', 'evaluated',
                                      'selected', 'appointed', 'rejected', 'waitlist')),
    reject_reason    TEXT,
    waitlist_rank    BIGINT,
    composite_score  REAL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workforce_cases_stage_county
    ON workforce_recruitment_cases (stage, county);
CREATE INDEX IF NOT EXISTS idx_workforce_cases_capacity
    ON workforce_recruitment_cases (county, role_code, stage);

-- Attributed transition history.
CREATE TABLE IF NOT EXISTS workforce_stage_history (
    history_id UUID PRIMARY KEY,
    case_id    UUID        NOT NULL REFERENCES workforce_recruitment_cases (case_id),
    from_stage TEXT        NOT NULL,
    to_stage   TEXT        NOT NULL,
    actor      TEXT        NOT NULL,
    at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason     TEXT
);

CREATE INDEX IF NOT EXISTS idx_workforce_stage_history_case ON workforce_stage_history (case_id, at);

-- ---------------------------------------------------------------------------
-- S5: Posting orders & equipment claims
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS workforce_postings (
    posting_id     UUID PRIMARY KEY,
    worker_id      UUID        NOT NULL REFERENCES workforce_workers (worker_id),
    station_code   TEXT        NOT NULL,
    role_code      TEXT        NOT NULL,
    effective_from TIMESTAMPTZ NOT NULL,
    effective_to   TIMESTAMPTZ,
    posted_by      TEXT        NOT NULL,
    superseded_by  UUID REFERENCES workforce_postings (posting_id),
    status         TEXT        NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'superseded')),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workforce_postings_worker ON workforce_postings (worker_id, status);
CREATE INDEX IF NOT EXISTS idx_workforce_postings_station ON workforce_postings (station_code);

CREATE TABLE IF NOT EXISTS workforce_equipment_claims (
    claim_id     UUID PRIMARY KEY,
    posting_id   UUID        NOT NULL REFERENCES workforce_postings (posting_id),
    kit_serials  JSONB       NOT NULL DEFAULT '[]',
    acknowledged BOOLEAN     NOT NULL DEFAULT FALSE,
    returned     BOOLEAN     NOT NULL DEFAULT FALSE,
    loss_flags   JSONB       NOT NULL DEFAULT '[]',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- At most one open (unreturned) claim per posting.
CREATE UNIQUE INDEX IF NOT EXISTS idx_workforce_claims_one_open
    ON workforce_equipment_claims (posting_id) WHERE returned = FALSE;

-- ---------------------------------------------------------------------------
-- S6: Tasking
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS workforce_task_templates (
    template_id UUID PRIMARY KEY,
    role_code   TEXT        NOT NULL,
    phase       TEXT        NOT NULL CHECK (phase IN ('pre_poll', 'polling_day', 'post_poll')),
    title       TEXT        NOT NULL,
    critical    BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workforce_task_templates_role_phase
    ON workforce_task_templates (role_code, phase);

CREATE TABLE IF NOT EXISTS workforce_task_assignments (
    assignment_id UUID PRIMARY KEY,
    posting_id    UUID        NOT NULL REFERENCES workforce_postings (posting_id),
    template_id   UUID        NOT NULL REFERENCES workforce_task_templates (template_id),
    due_from      TIMESTAMPTZ NOT NULL,
    due_to        TIMESTAMPTZ NOT NULL,
    status        TEXT        NOT NULL DEFAULT 'pending'
                  CHECK (status IN ('pending', 'acked', 'done', 'blocked')),
    evidence_hash CHAR(64),
    block_reason  TEXT,
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workforce_assignments_posting_status
    ON workforce_task_assignments (posting_id, status);

-- ---------------------------------------------------------------------------
-- S7: Performance (append-only)
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS workforce_performance_records (
    record_id   UUID PRIMARY KEY,
    worker_id   UUID        NOT NULL REFERENCES workforce_workers (worker_id),
    category    TEXT        NOT NULL CHECK (category IN ('attendance', 'punctuality', 'incident', 'evaluation')),
    score       REAL,
    detail      TEXT        NOT NULL,
    recorded_by TEXT        NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workforce_perf_worker ON workforce_performance_records (worker_id, recorded_at);
