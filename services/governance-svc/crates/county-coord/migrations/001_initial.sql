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

-- M56 county-national-coordination schema
CREATE TABLE IF NOT EXISTS mous (
    id              TEXT PRIMARY KEY,
    county_code     TEXT NOT NULL,
    ministry_code   TEXT,
    party_type      TEXT NOT NULL CHECK (party_type IN ('County','Ministry')),
    effective_date  TEXT NOT NULL,
    expiry_date     TEXT NOT NULL,
    content_url     TEXT,
    status          TEXT NOT NULL DEFAULT 'Draft' CHECK (status IN ('Draft','Signed','Active','Expired')),
    signed_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS county_datasets (
    id              TEXT PRIMARY KEY,
    county_code     TEXT NOT NULL,
    dataset_type    TEXT NOT NULL CHECK (dataset_type IN ('Population','SchoolFacility','HealthFacility','RoadAccess','SecurityHistory')),
    source_system   TEXT NOT NULL,
    approved_by     TEXT,
    approval_date   TEXT,
    dataset_url     TEXT,
    schema_version  TEXT,
    lawful_basis_code TEXT NOT NULL,
    purpose_code    TEXT NOT NULL,
    retention_until TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ministry_approvals (
    id              TEXT PRIMARY KEY,
    mou_id          TEXT NOT NULL REFERENCES mous(id),
    ministry_ref    TEXT NOT NULL,
    approved_by     TEXT NOT NULL,
    approved_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS coordination_incidents (
    id              TEXT PRIMARY KEY,
    county_code     TEXT NOT NULL,
    incident_type   TEXT NOT NULL,
    severity        TEXT NOT NULL CHECK (severity IN ('Low','Medium','High','Critical')),
    description     TEXT,
    reported_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at     TIMESTAMPTZ,
    ecc_alert_ref   TEXT
);

CREATE INDEX IF NOT EXISTS idx_incidents_severity ON coordination_incidents(severity);
CREATE INDEX IF NOT EXISTS idx_mous_county ON mous(county_code);
CREATE INDEX IF NOT EXISTS idx_mous_expiry ON mous(expiry_date);
