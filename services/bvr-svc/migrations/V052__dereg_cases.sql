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

-- V052: deregistration cases and appeals (WP-4.10, M36)
-- Columns mirror dereg-appeal/src/models.rs DeregistrationCase and AppealRecord.
-- Enum values are stored SCREAMING_SNAKE_CASE, matching the serde renaming
-- used by CaseStatus, AdjudicatorDecision, IntakeChannel, and AppealGrounds.

CREATE TABLE IF NOT EXISTS dereg_cases (
    id                        UUID        NOT NULL PRIMARY KEY,
    voter_id                  UUID        NOT NULL,
    m35_event_id              UUID        NOT NULL,
    status                    TEXT        NOT NULL
        CHECK (status IN ('NOTICED','APPEAL_FILED','UNDER_REVIEW','ADJUDICATION',
                          'OVERTURNED','CONFIRMED','DEREGISTERED','EXPIRED_NO_APPEAL')),
    notice_dispatched_at      TIMESTAMPTZ,
    notice_deadline           TIMESTAMPTZ,
    appeal_id                 UUID,
    officer_id                UUID,
    officer_recommendation    TEXT,
    adjudicator_id            UUID,
    adjudicator_decision      TEXT
        CHECK (adjudicator_decision IS NULL OR adjudicator_decision IN ('OVERTURN','CONFIRM')),
    adjudicator_decision_at   TIMESTAMPTZ,
    adjudicator_notes         TEXT,
    data_minimisation_at      TIMESTAMPTZ,
    data_minimisation_officer UUID,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dereg_cases_voter  ON dereg_cases (voter_id);
CREATE INDEX IF NOT EXISTS idx_dereg_cases_status ON dereg_cases (status);

CREATE TABLE IF NOT EXISTS dereg_appeals (
    id                          UUID        NOT NULL PRIMARY KEY,
    case_id                     UUID        NOT NULL REFERENCES dereg_cases (id),
    intake_channel              TEXT        NOT NULL
        CHECK (intake_channel IN ('VERIFIER_APP','WEB','USSD','IN_PERSON')),
    appellant_id_hash           TEXT        NOT NULL,
    representative_auth_file_id UUID,
    grounds                     TEXT        NOT NULL
        CHECK (grounds IN ('IDENTITY_ERROR','ALIVE_AND_PRESENT','DATA_MATCH_ERROR',
                           'FAMILY_DISPUTE','OTHER')),
    grounds_detail              TEXT        NOT NULL,
    supporting_doc_ids          UUID[]      NOT NULL DEFAULT '{}',
    language                    TEXT        NOT NULL,
    filed_at                    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    hearing_date                TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_dereg_appeals_case ON dereg_appeals (case_id);
