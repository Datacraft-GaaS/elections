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

-- Per-job quality reports: one row per quality check run.
-- Stores per-modality scores and retake reasons.

CREATE TABLE IF NOT EXISTS eel_quality_reports (
    id                  UUID        PRIMARY KEY,
    enrolment_job_id    UUID        NOT NULL REFERENCES eel_enrolment_jobs(id),
    per_modality        JSONB        NOT NULL DEFAULT '[]',
    usable_finger_count SMALLINT    NOT NULL DEFAULT 0,
    iris_captured       BOOLEAN     NOT NULL DEFAULT FALSE,
    face_captured       BOOLEAN     NOT NULL DEFAULT FALSE,
    retake_reasons      JSONB        NOT NULL DEFAULT '[]',
    checked_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_eel_quality_reports_job
    ON eel_quality_reports (enrolment_job_id);
