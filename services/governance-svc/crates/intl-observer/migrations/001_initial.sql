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

-- M57 international-observer-coordination schema
CREATE TABLE IF NOT EXISTS observer_missions (
    id                  TEXT PRIMARY KEY,
    organisation_name   TEXT NOT NULL,
    accreditation_ref   TEXT NOT NULL,
    lead_contact_hash   TEXT NOT NULL,
    workspace_id        TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'Registered' CHECK (status IN ('Registered','Active','Closed')),
    registered_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS briefings (
    id                  TEXT PRIMARY KEY,
    mission_id          TEXT NOT NULL REFERENCES observer_missions(id),
    topic               TEXT NOT NULL,
    scheduled_at        TIMESTAMPTZ NOT NULL,
    materials_url       TEXT,
    attendance_confirmed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS observation_reports (
    id          TEXT PRIMARY KEY,
    mission_id  TEXT NOT NULL REFERENCES observer_missions(id),
    report_type TEXT NOT NULL CHECK (report_type IN ('interim','final')),
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    content_url TEXT NOT NULL,
    classified  BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS joint_statements (
    id                      TEXT PRIMARY KEY,
    participating_mission_ids TEXT NOT NULL,
    published_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    content_url             TEXT NOT NULL,
    signed_by               TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_missions_status ON observer_missions(status);
CREATE INDEX IF NOT EXISTS idx_briefings_mission ON briefings(mission_id);
CREATE INDEX IF NOT EXISTS idx_reports_mission ON observation_reports(mission_id);
