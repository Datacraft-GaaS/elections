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

-- G23 — Section 44A Complementary Mechanism Drill (CLOSED)
-- Migration creates procedural lesson P7: Escalation Protocol & Complementary Mechanism
-- References: Elections Act §44A, ADR-0019 (security), ADR-0023 (audit)
-- Lesson P7 covers: escalation ladder, manual form collection, rapid-response activation, CCB convening
INSERT INTO lms_app_lesson (id, module_id, title, order_in_module, audit_ref, proof_hash, created_at)
VALUES (7, 6, 'Section 44A Escalation & Complementary Mechanism', 7,
        'g23-section44a-lesson-p7',
        'sha256(section44a-procedural-lesson-v2)',
        NOW());
