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

-- M58/M59 Parliamentary/Judiciary Reporting and OAG Interface tables (WP-5.1)

CREATE TABLE reporting_statutory_reports (
    report_id           TEXT PRIMARY KEY,
    report_type         TEXT NOT NULL CHECK (report_type IN ('Assembly','Senate','Judiciary','SC','Committee')),
    status              TEXT NOT NULL DEFAULT 'queued'
                            CHECK (status IN ('queued','generating','ready','delivered')),
    period_from         DATE NOT NULL,
    period_to           DATE NOT NULL,
    output_url          TEXT,
    sig                 TEXT,
    generated_at        TIMESTAMPTZ,
    delivered_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE reporting_testimony_workspaces (
    workspace_id        TEXT PRIMARY KEY,
    report_id           TEXT NOT NULL REFERENCES reporting_statutory_reports(report_id),
    committee_name      TEXT NOT NULL,
    testimony_date      DATE,
    notes               TEXT,
    status              TEXT NOT NULL DEFAULT 'draft'
                            CHECK (status IN ('draft','finalised','submitted')),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE reporting_judiciary_handovers (
    handover_id         TEXT PRIMARY KEY,
    report_id           TEXT NOT NULL REFERENCES reporting_statutory_reports(report_id),
    handed_to           TEXT NOT NULL,
    handover_ref        TEXT,
    signed_at           TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE reporting_sc_preregistrations (
    prereg_id           TEXT PRIMARY KEY,
    contact_name        TEXT NOT NULL,
    contact_email_hash  TEXT NOT NULL,
    system_name         TEXT NOT NULL,
    registered_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE oag_engagements (
    engagement_id       TEXT PRIMARY KEY,
    title               TEXT NOT NULL,
    scope               TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'scoping'
                            CHECK (status IN ('scoping','active','responded','closed')),
    oag_ref             TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE oag_evidence_extracts (
    extract_id          TEXT PRIMARY KEY,
    engagement_id       TEXT NOT NULL REFERENCES oag_engagements(engagement_id),
    query_hash          TEXT NOT NULL,
    extracted_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    row_count           INTEGER,
    output_url          TEXT
);

CREATE TABLE oag_responses (
    response_id         TEXT PRIMARY KEY,
    engagement_id       TEXT NOT NULL REFERENCES oag_engagements(engagement_id),
    response_text       TEXT NOT NULL,
    responded_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    responder_hash      TEXT NOT NULL
);

CREATE TABLE oag_published_reports (
    pub_report_id       TEXT PRIMARY KEY,
    engagement_id       TEXT REFERENCES oag_engagements(engagement_id),
    title               TEXT NOT NULL,
    published_at        DATE NOT NULL,
    url                 TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_statutory_reports_type ON reporting_statutory_reports(report_type, status);
CREATE INDEX idx_oag_engagements_status ON oag_engagements(status);
