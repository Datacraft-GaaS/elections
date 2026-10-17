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

-- M37 CMS: bring storage into line with the application's models.
--
-- Same direction as 0007 and for the same reason: `training-api` declares no
-- database driver, so nothing outside the Django app reads `training_cms_*`
-- and the app owns the database outright. The model is the schema of record.
--
-- Most of the difference here is naming. The table was written with
-- abbreviated column names (`item_id`, `version_num`, `body`) and the
-- application was written against the descriptive ones (`content_item_id`,
-- `version_number`, `body_markdown`); the application code, the admin and the
-- tests all use the model's spelling, so the columns are renamed rather than
-- duplicated. `ALTER TABLE ... RENAME COLUMN` keeps the type, the constraints
-- and the data, which a drop-and-add would not.
--
-- `created_by` is renamed to `author_id` only on `training_cms_content_items`,
-- where the model calls the author `author_id`. On
-- `training_cms_content_versions` the model calls it `created_by` and the
-- table already agrees, so it is untouched.

-- ---------------------------------------------------------------------------
-- training_cms_content_items
-- ---------------------------------------------------------------------------

ALTER TABLE training_cms_content_items RENAME COLUMN created_by TO author_id;

ALTER TABLE training_cms_content_items
    ADD COLUMN IF NOT EXISTS current_version_id UUID,
    ADD COLUMN IF NOT EXISTS approved_by        UUID,
    ADD COLUMN IF NOT EXISTS approved_at        TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_cms_items_author   ON training_cms_content_items (author_id);
CREATE INDEX IF NOT EXISTS idx_cms_items_approved ON training_cms_content_items (approved_at);

-- ---------------------------------------------------------------------------
-- training_cms_content_versions
-- ---------------------------------------------------------------------------

ALTER TABLE training_cms_content_versions RENAME COLUMN item_id     TO content_item_id;
ALTER TABLE training_cms_content_versions RENAME COLUMN version_num TO version_number;
ALTER TABLE training_cms_content_versions RENAME COLUMN body        TO body_markdown;
ALTER TABLE training_cms_content_versions RENAME COLUMN asset_url   TO asset_file_id;

-- `asset_url` held a URL and the model types the field as a UUID reference to
-- an asset record; the rename is a change of meaning, so the column is
-- retyped rather than left as text that happens to be named `asset_file_id`.
ALTER TABLE training_cms_content_versions
    ALTER COLUMN asset_file_id TYPE UUID USING NULL;

ALTER TABLE training_cms_content_versions
    ADD COLUMN IF NOT EXISTS parent_version_id UUID;

-- `version_num` was SMALLINT and the model's `PositiveIntegerField` is
-- `integer`. A rename keeps the type, so the widening has to be explicit;
-- leaving it would raise on the first version number past 32767 and the gate
-- reports the mismatch as drift.
ALTER TABLE training_cms_content_versions
    ALTER COLUMN version_number TYPE INTEGER USING version_number::integer;

CREATE INDEX IF NOT EXISTS idx_cms_versions_item ON training_cms_content_versions (content_item_id);

-- ---------------------------------------------------------------------------
-- training_cms_fact_check_reviews
-- ---------------------------------------------------------------------------

ALTER TABLE training_cms_fact_check_reviews RENAME COLUMN item_id     TO content_item_id;
ALTER TABLE training_cms_fact_check_reviews RENAME COLUMN reviewed_at TO completed_at;

ALTER TABLE training_cms_fact_check_reviews
    ADD COLUMN IF NOT EXISTS version_id UUID;

CREATE INDEX IF NOT EXISTS idx_cms_factcheck_item ON training_cms_fact_check_reviews (content_item_id);

-- ---------------------------------------------------------------------------
-- training_cms_distribution_events
-- ---------------------------------------------------------------------------

ALTER TABLE training_cms_distribution_events RENAME COLUMN item_id        TO content_item_id;
ALTER TABLE training_cms_distribution_events RENAME COLUMN distributed_at TO dispatched_at;

ALTER TABLE training_cms_distribution_events
    ADD COLUMN IF NOT EXISTS quantity INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_cms_distribution_item
    ON training_cms_distribution_events (content_item_id);
