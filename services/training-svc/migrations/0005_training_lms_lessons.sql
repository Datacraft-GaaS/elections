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

-- M38: lesson-level LMS schema — courses, lessons, attempts, certificate audit.
-- migration: 0005_training_lms_lessons
--
-- Extends 0002_training_lms (modules, enrollments, certificates, progress) to
-- the lesson grain. Column names match the Django models in
-- django/lms/lms_app/models.py, whose Meta.db_table names these tables, so the
-- ORM and the database agree on one schema rather than two.
--
-- Lives at services/training-svc/migrations/ because that is the only path
-- tools/eel-migrate discovers; a copy under django/lms/ is never applied.

CREATE TABLE training_lms_courses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    version         INTEGER NOT NULL DEFAULT 1,
    published_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- A lesson is drawn from exactly one module (0002) and ordered within one
-- course. `UNIQUE (course_id, order_index)` is what makes the ordering a
-- sequence rather than a set of numbers that happen to be distinct.
CREATE TABLE training_lms_lessons (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id           UUID NOT NULL REFERENCES training_lms_courses (id) ON DELETE CASCADE,
    module_id           UUID NOT NULL REFERENCES training_lms_modules (id) ON DELETE CASCADE,
    order_index         SMALLINT NOT NULL DEFAULT 0,
    title               TEXT NOT NULL,
    content_markdown    TEXT NOT NULL,
    duration_minutes    SMALLINT NOT NULL DEFAULT 15,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (course_id, order_index)
);

CREATE INDEX idx_lms_lessons_course_id ON training_lms_lessons (course_id);
CREATE INDEX idx_lms_lessons_module_id ON training_lms_lessons (module_id);

-- `idempotency_key` is UNIQUE so a retried offline sync cannot double-count an
-- attempt; the worker writes the same key on replay (M38 offline-first).
CREATE TABLE training_lms_lesson_attempts (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    worker_id           UUID NOT NULL,
    lesson_id           UUID NOT NULL REFERENCES training_lms_lessons (id) ON DELETE CASCADE,
    attempted_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed           BOOLEAN NOT NULL DEFAULT false,
    score_pct           NUMERIC(5,2),
    idempotency_key     TEXT NOT NULL UNIQUE
);

CREATE INDEX idx_lms_attempts_worker_id ON training_lms_lesson_attempts (worker_id);
CREATE INDEX idx_lms_attempts_lesson_id ON training_lms_lesson_attempts (lesson_id);

-- M5 audit chain for certificate lifecycle events. proof_hash and
-- correlation_key are the M5 lattice fields; ADR-0023 requires both on every
-- certificate event.
CREATE TABLE training_lms_cert_audit (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type          TEXT NOT NULL,
    certificate_id      UUID,
    worker_id           UUID NOT NULL,
    recorded_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    proof_hash          TEXT NOT NULL,
    correlation_key     TEXT NOT NULL
);

CREATE INDEX idx_lms_cert_audit_worker_id ON training_lms_cert_audit (worker_id);
CREATE INDEX idx_lms_cert_audit_event_type ON training_lms_cert_audit (event_type);
