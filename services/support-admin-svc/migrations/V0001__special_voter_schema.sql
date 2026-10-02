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

-- support-admin-svc: special voter and intent domain tables (WP-5.5, M25, M27).
-- Applied by sqlx migrate on service startup.

-- ---- Special Voter Facilities (M25) ----------------------------------------

CREATE TABLE IF NOT EXISTS eel_special_voter_facilities (
    id                      UUID PRIMARY KEY,
    category                TEXT NOT NULL CHECK (category IN (
                                'REMAND_PRISON','HOSPITAL','BARRACKS',
                                'REFUGEE_CAMP','IDP_CAMP')),
    county_code             TEXT NOT NULL,
    kit_ids                 TEXT[] NOT NULL DEFAULT '{}',
    presiding_officer_id    TEXT NOT NULL,
    languages               TEXT[] NOT NULL DEFAULT '{en,sw}',
    voting_date             DATE NOT NULL,
    published_voter_count   INTEGER NOT NULL DEFAULT 0,
    session_state           TEXT NOT NULL DEFAULT 'SEALED'
                                CHECK (session_state IN ('SEALED','DUAL_BIOMETRIC_PENDING',
                                'OPEN','SUSPENDED','CLOSED')),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_svf_category ON eel_special_voter_facilities (category);
CREATE INDEX IF NOT EXISTS idx_svf_county ON eel_special_voter_facilities (county_code);
CREATE INDEX IF NOT EXISTS idx_svf_date ON eel_special_voter_facilities (voting_date);

-- ---- Special Voter Events (M25, audit class A) ------------------------------

CREATE TABLE IF NOT EXISTS eel_special_voter_events (
    id                  UUID PRIMARY KEY,
    facility_id         UUID NOT NULL REFERENCES eel_special_voter_facilities(id),
    category            TEXT NOT NULL,
    event_type          TEXT NOT NULL,
    -- voter_id encrypted at rest; never stored plaintext.
    voter_id_enc        BYTEA,
    officer_id          TEXT NOT NULL,
    witness_ids         TEXT[] NOT NULL DEFAULT '{}',
    reason_code         TEXT,
    merkle_node_id      TEXT,
    event_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_sve_facility ON eel_special_voter_events (facility_id);
CREATE INDEX IF NOT EXISTS idx_sve_event_type ON eel_special_voter_events (event_type);

-- ---- Special Form 34 variants (M25) ----------------------------------------

CREATE TABLE IF NOT EXISTS eel_special_form34 (
    id                  UUID PRIMARY KEY,
    facility_id         UUID NOT NULL REFERENCES eel_special_voter_facilities(id),
    form_variant        TEXT NOT NULL CHECK (form_variant IN ('34A-R','34A-H','34A-B','34A-I')),
    tallies             JSONB NOT NULL DEFAULT '{}',
    signatures          JSONB NOT NULL DEFAULT '[]',
    transmitted_at      TIMESTAMPTZ,
    merkle_node_id      TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---- IDP Reassignments (M25) ------------------------------------------------

CREATE TABLE IF NOT EXISTS eel_idp_reassignments (
    id                      UUID PRIMARY KEY,
    -- voter_id encrypted at rest (DPA 2019 compliance).
    voter_id_enc            BYTEA NOT NULL,
    original_station_code   TEXT NOT NULL,
    idp_facility_id         UUID NOT NULL REFERENCES eel_special_voter_facilities(id),
    displacement_source     TEXT NOT NULL,
    effective_date          DATE NOT NULL,
    reassigned_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_idp_facility ON eel_idp_reassignments (idp_facility_id);

-- ---- S.44A Override log (M25 guard audit) -----------------------------------

CREATE TABLE IF NOT EXISTS eel_s44a_overrides (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    facility_id                 UUID NOT NULL REFERENCES eel_special_voter_facilities(id),
    reason_code                 TEXT NOT NULL,
    requesting_officer_id       TEXT NOT NULL,
    countersigning_officer_id   TEXT NOT NULL,
    approved                    BOOLEAN NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);
