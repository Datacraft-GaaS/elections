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

-- G23 — Section 44A Complementary Mechanism Drill
-- migration: 0006_section_44a_drill
--
-- Adds procedural lesson P7 to the M38 course: the escalation ladder, manual
-- form collection, rapid-response activation and CCB convening. References:
-- Elections Act §44A, ADR-0019 (security), ADR-0023 (audit).
--
-- The lesson hangs off the course and module the 0005 seed creates, both of
-- which are inserted here first so this file stands on its own when it runs
-- against an empty database. `order_index` 11 continues the 1..10 sequence the
-- seed establishes; `UNIQUE (course_id, order_index)` rejects a collision.
--
-- The previous version inserted into `lms_app_lesson` with columns
-- `order_in_module`, `audit_ref` and `proof_hash`, none of which exist in any
-- schema this repository creates.

INSERT INTO training_lms_courses (id, title, version, published_at) VALUES
('a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'Poll Worker Certification (M38)', 1, '2025-01-15 08:00:00')
ON CONFLICT (id) DO NOTHING;

INSERT INTO training_lms_modules (id, title, target_audience, module_type, pass_mark_pct) VALUES
('11111111-1111-4111-8111-111111111106', 'Closing & Reconciliation', 'PO,DPO', 'CORE', 82)
ON CONFLICT (id) DO NOTHING;

INSERT INTO training_lms_lessons (id, course_id, module_id, order_index, title, content_markdown, duration_minutes) VALUES
('22222222-2222-4222-8222-22222222220b',
 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
 '11111111-1111-4111-8111-111111111106',
 11,
 'P7: Section 44A Escalation & Complementary Mechanism',
 'Escalation ladder, manual form collection, rapid-response activation, CCB convening. Elections Act §44A.',
 30)
ON CONFLICT (id) DO NOTHING;

-- M5 audit chain entry for the lesson's publication, so the drill lesson is
-- traceable the way every other certificate-lifecycle event is (ADR-0023).
INSERT INTO training_lms_cert_audit (id, event_type, worker_id, proof_hash, correlation_key) VALUES
('55555555-5555-4555-8555-555555555502',
 'LESSON_PUBLISHED',
 '44444444-4444-4444-8444-444444444401',
 'sha256(section44a-procedural-lesson-v2)',
 'g23-section44a-lesson-p7')
ON CONFLICT (id) DO NOTHING;
