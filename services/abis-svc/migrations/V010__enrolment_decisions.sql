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

-- Final enrolment decisions: written once a job reaches Enrolled state.
-- Links back to the enrolment job; records gallery placement + merge metadata.

CREATE TABLE IF NOT EXISTS eel_enrolment_decisions (
    id                  UUID        PRIMARY KEY,
    enrolment_job_id    UUID        NOT NULL REFERENCES eel_enrolment_jobs(id),
    subject_id          UUID        NOT NULL,
    enrolled_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    gallery_shard_id    TEXT        NOT NULL,
    previous_records    JSONB        NOT NULL DEFAULT '[]',
    merge_from          UUID,
    adjudicator_id      UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_enrolment_decision_job
    ON eel_enrolment_decisions (enrolment_job_id);

CREATE INDEX IF NOT EXISTS idx_eel_enrolment_decisions_subject
    ON eel_enrolment_decisions (subject_id);
