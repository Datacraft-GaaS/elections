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

-- M60: Medical First Response
-- migration: 0004_training_medical
-- No PHI fields: patient name, diagnosis, or health identifiers are prohibited by FR-M60-011

CREATE TABLE training_medical_incidents (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    station_id      TEXT NOT NULL,
    incident_type   TEXT NOT NULL,                          -- cardiac, trauma, heat_exhaustion, other
    severity        TEXT NOT NULL,                          -- low, medium, high, critical
    status          TEXT NOT NULL DEFAULT 'open',           -- open, dispatched, resolved
    persons_affected INTEGER NOT NULL DEFAULT 0,
    county_code     TEXT,
    reported_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at     TIMESTAMPTZ
    -- No PHI fields: patient name, diagnosis, or health identifiers are prohibited by FR-M60-011
);

CREATE INDEX idx_medical_incidents_status      ON training_medical_incidents (status);
CREATE INDEX idx_medical_incidents_county      ON training_medical_incidents (county_code);
CREATE INDEX idx_medical_incidents_reported_at ON training_medical_incidents (reported_at);

CREATE TABLE training_medical_dispatch_requests (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id     UUID NOT NULL REFERENCES training_medical_incidents (id),
    dispatched_to   TEXT NOT NULL,                          -- KRCS, MoH, NDMU
    status          TEXT NOT NULL DEFAULT 'pending',        -- pending, en_route, on_scene
    dispatched_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_medical_dispatch_incident_id ON training_medical_dispatch_requests (incident_id);

CREATE TABLE training_medical_cluster_alerts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    county_code     TEXT NOT NULL,
    incident_count  INTEGER NOT NULL,
    alert_threshold INTEGER NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active',         -- active, resolved
    raised_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at     TIMESTAMPTZ
);

CREATE INDEX idx_medical_cluster_alerts_county  ON training_medical_cluster_alerts (county_code);
CREATE INDEX idx_medical_cluster_alerts_status  ON training_medical_cluster_alerts (status);

CREATE TABLE training_medical_units (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    unit_name       TEXT NOT NULL,
    provider        TEXT NOT NULL,                          -- KRCS, MoH, NDMU
    county_code     TEXT NOT NULL,
    station_id      TEXT,
    status          TEXT NOT NULL DEFAULT 'standby',        -- standby, deployed, unavailable
    registered_at   TIMESTAMPTZ NOT NULL DEFAULT now()
    -- No PHI fields: patient name, diagnosis, or health identifiers are prohibited by FR-M60-011
);

CREATE INDEX idx_medical_units_county  ON training_medical_units (county_code);
CREATE INDEX idx_medical_units_status  ON training_medical_units (status);
