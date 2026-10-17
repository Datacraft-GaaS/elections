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

-- M38 LMS: bring storage into line with the application's models.
--
-- Direction. The rule for this reconciliation is SQL wins where an app shares
-- a database with a Rust service, models win where the app owns the database
-- outright. `training-svc/crates/training-api` declares no database driver at
-- all -- its `store.rs` is an in-memory `Arc<RwLock<...>>` and the crate has
-- no sqlx, tokio-postgres, diesel or sea-orm dependency -- so nothing outside
-- the Django app reads or writes `training_lms_*`. The app owns the database
-- outright, so the model is the schema of record and this migration adds the
-- columns the models declare.
--
-- Why renames rather than drop-and-add: where both sides name the same
-- concept (`attempt_count` is what the model calls `attempts`), the rename
-- preserves the column's type, constraints and any data, and the application
-- code and tests already use the model's spelling.
--
-- `best_score_pct` is the one column that is not a pure rename. The table
-- stored `last_score_pct` -- the most recent attempt -- while the model and
-- `views.WorkerEnrollmentViewSet.submit_assessment` maintain a best-so-far
-- maximum (`if score_pct > enrollment.best_score_pct`). The column is renamed
-- and widened to the model's type, because a SMALLINT could not hold the
-- Decimal the view writes.
--
-- The drift gate (`tools/check-django-schema-drift.py`) compares model columns
-- against a database built from this chain, so it verifies this migration.

-- ---------------------------------------------------------------------------
-- training_lms_modules
-- ---------------------------------------------------------------------------

ALTER TABLE training_lms_modules
    ADD COLUMN IF NOT EXISTS language         VARCHAR(20) NOT NULL DEFAULT 'en',
    ADD COLUMN IF NOT EXISTS content_markdown TEXT        NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS asset_ids        JSONB       NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS case_study_ids   JSONB       NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS max_retakes      SMALLINT    NOT NULL DEFAULT 3,
    ADD COLUMN IF NOT EXISTS version          INTEGER     NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS expiry_date      TIMESTAMPTZ;

-- The `language` filter in `CourseModuleViewSet.filterset_fields` reads this.
CREATE INDEX IF NOT EXISTS idx_lms_modules_language ON training_lms_modules (language);

-- ---------------------------------------------------------------------------
-- training_lms_enrollments
-- ---------------------------------------------------------------------------

ALTER TABLE training_lms_enrollments RENAME COLUMN attempt_count TO attempts;
ALTER TABLE training_lms_enrollments RENAME COLUMN last_score_pct TO best_score_pct;

ALTER TABLE training_lms_enrollments
    ALTER COLUMN best_score_pct TYPE NUMERIC(5, 2) USING best_score_pct::numeric(5, 2),
    ALTER COLUMN best_score_pct SET DEFAULT 0,
    ALTER COLUMN best_score_pct SET NOT NULL;

-- `certified` is what the certificate gate and the admin filter read; the
-- table had no equivalent, so this is an addition rather than a rename.
ALTER TABLE training_lms_enrollments
    ADD COLUMN IF NOT EXISTS certified BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX IF NOT EXISTS idx_lms_enrollments_certified
    ON training_lms_enrollments (certified);

-- ---------------------------------------------------------------------------
-- training_lms_certificates
-- ---------------------------------------------------------------------------

ALTER TABLE training_lms_certificates
    ADD COLUMN IF NOT EXISTS station_code      VARCHAR(20) NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS modules_completed JSONB       NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS revoked_by        UUID;

-- ---------------------------------------------------------------------------
-- training_lms_progress
-- ---------------------------------------------------------------------------

ALTER TABLE training_lms_progress RENAME COLUMN certified_count TO certified;
ALTER TABLE training_lms_progress RENAME COLUMN refresher_pending_count TO refresher_pending;
ALTER TABLE training_lms_progress RENAME COLUMN not_certified_count TO not_certified;

ALTER TABLE training_lms_progress
    ADD COLUMN IF NOT EXISTS certification_rate_pct NUMERIC(5, 2) NOT NULL DEFAULT 0;
