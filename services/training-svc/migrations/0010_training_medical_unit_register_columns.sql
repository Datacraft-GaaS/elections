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

-- M60: relax the facility-register columns the application does not supply.
--
-- 0009 added the model's columns to `training_medical_units` but left the
-- register columns (`unit_name`, `provider`, `county_code`) NOT NULL. The
-- model has no field for any of the three and the application is the only
-- writer, so `MedicalUnit.objects.create(...)` fails on the not-null
-- constraint before it ever reaches a column the model knows about.
--
-- A probe driving the ORM against a database built from this chain found it:
--
--   IntegrityError: null value in column "unit_name" of relation
--   "training_medical_units" violates not-null constraint
--
-- The drift gate cannot see this class of defect. It compares the model's
-- columns against the table's and reports a model column the table lacks; a
-- table column the model lacks is not drift, so a NOT NULL column nothing can
-- supply reads as agreement.
--
-- The constraint was written for a facility register -- an object the
-- application does not implement. Making the columns nullable lets both
-- coexist: the model's writes succeed, and the register columns remain for
-- the dispatch flow `docs/CAPABILITIES.md` describes ("nearest KRCS unit in
-- training_medical_units with status: standby and matching county_code"),
-- which is not implemented in `medical_app`.

ALTER TABLE training_medical_units
    ALTER COLUMN unit_name   DROP NOT NULL,
    ALTER COLUMN provider    DROP NOT NULL,
    ALTER COLUMN county_code DROP NOT NULL;

COMMENT ON COLUMN training_medical_units.unit_name IS
    'Facility register: display name. Nullable because the application models a '
    'geolocated capability record and never supplies it.';
COMMENT ON COLUMN training_medical_units.provider IS
    'Facility register: operating organisation (KRCS/MoH/NDMU). Nullable, see unit_name.';
COMMENT ON COLUMN training_medical_units.county_code IS
    'Facility register: county code (KE-047). Distinct from `county`, which holds '
    'a name. Nullable, see unit_name.';
