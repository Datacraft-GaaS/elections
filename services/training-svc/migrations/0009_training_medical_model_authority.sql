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

-- M60 Medical first response: bring storage into line with the models.
--
-- Same direction as 0007 and 0008: `training-api` declares no database driver,
-- so the Django app owns `training_medical_*` outright and its models are the
-- schema of record.
--
-- `training_medical_units` is the substantive one. The table was created as a
-- facility register (`unit_name`, `provider`, `station_id`, `status`) while
-- the model and `views.MedicalUnitViewSet` address it as a geolocated
-- capability record (`unit_id`, `unit_type`, `county`, `sub_county`,
-- `gps_lat`, `gps_lon`, `contact_number`, `capacity`, `locked`). Those are two
-- different objects sharing a table name, so this adds the model's columns and
-- leaves the register columns in place rather than renaming across a change of
-- meaning. `unit_name` and `provider` stay NOT NULL and the new columns are
-- nullable or defaulted, so existing rows and existing inserts both survive.
--
-- `county` here is deliberately distinct from `county_code`: the register
-- column holds a code (`KE-047`) and the model's field holds a name. The
-- `county_code` column is left alone; a later revision should decide whether
-- the two should be joined.

-- ---------------------------------------------------------------------------
-- training_medical_incidents
-- ---------------------------------------------------------------------------

ALTER TABLE training_medical_incidents
    ADD COLUMN IF NOT EXISTS reporter_id UUID;

CREATE INDEX IF NOT EXISTS idx_medical_incidents_reporter
    ON training_medical_incidents (reporter_id);

-- ---------------------------------------------------------------------------
-- training_medical_dispatch_requests
-- ---------------------------------------------------------------------------

ALTER TABLE training_medical_dispatch_requests
    ADD COLUMN IF NOT EXISTS eta_minutes     SMALLINT,
    ADD COLUMN IF NOT EXISTS acknowledged_at TIMESTAMPTZ;

-- ---------------------------------------------------------------------------
-- training_medical_cluster_alerts
-- ---------------------------------------------------------------------------

-- The model has no `status` column and the table has no `resolved_at`-only
-- lifecycle. `status` is nullable on the table already, so nothing here.

-- ---------------------------------------------------------------------------
-- training_medical_units
-- ---------------------------------------------------------------------------

ALTER TABLE training_medical_units
    ADD COLUMN IF NOT EXISTS unit_id        VARCHAR(50),
    ADD COLUMN IF NOT EXISTS unit_type      VARCHAR(30),
    ADD COLUMN IF NOT EXISTS county         VARCHAR(30),
    ADD COLUMN IF NOT EXISTS sub_county     VARCHAR(50),
    ADD COLUMN IF NOT EXISTS gps_lat        NUMERIC(9, 6),
    ADD COLUMN IF NOT EXISTS gps_lon        NUMERIC(9, 6),
    ADD COLUMN IF NOT EXISTS contact_number VARCHAR(20),
    ADD COLUMN IF NOT EXISTS capacity       SMALLINT NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS locked         BOOLEAN  NOT NULL DEFAULT FALSE;

-- The model declares `unit_id` unique, and `MedicalUnit` is the only writer.
CREATE UNIQUE INDEX IF NOT EXISTS uq_medical_units_unit_id
    ON training_medical_units (unit_id) WHERE unit_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_medical_units_county
    ON training_medical_units (county);
