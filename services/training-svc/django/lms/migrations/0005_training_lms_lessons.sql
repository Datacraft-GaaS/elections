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

CREATE TABLE IF NOT EXISTS training_lms_courses (id UUID PRIMARY KEY, title VARCHAR(500), description TEXT, version INT DEFAULT 1, published_at TIMESTAMPTZ);
CREATE TABLE IF NOT EXISTS training_lms_lessons (id UUID PRIMARY KEY, course_id UUID, module_id UUID, order_index INT DEFAULT 0, title VARCHAR(500), content_markdown TEXT, duration_minutes INT DEFAULT 15, UNIQUE(course_id, order_index));
CREATE TABLE IF NOT EXISTS training_lms_lesson_attempts (id UUID PRIMARY KEY, worker_id UUID, lesson_id UUID REFERENCES training_lms_lessons(id), attempted_at TIMESTAMPTZ DEFAULT now(), completed BOOLEAN DEFAULT FALSE, score_pct DECIMAL(5,2), idempotency_key VARCHAR(128) UNIQUE);
CREATE TABLE IF NOT EXISTS training_lms_cert_audit (id UUID PRIMARY KEY, event_type VARCHAR(30), certificate_id UUID, worker_id UUID, recorded_at TIMESTAMPTZ DEFAULT now(), proof_hash VARCHAR(64), correlation_key VARCHAR(64));
