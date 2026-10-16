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

-- M38 LMS seed: one course, six modules, ten lessons, three attempts, one audit row.
--
-- Run after services/training-svc/migrations/, whose schema this targets.
-- Columns are the ones 0002_training_lms and 0005_training_lms_lessons create:
-- `training_lms_modules` here has no `language` or `version` column, and
-- `training_lms_enrollments` names its counters `attempt_count` and
-- `last_score_pct`. Every id is a literal UUID so rows are stable across runs
-- and the foreign keys resolve by inspection; the previous version used slugs
-- ('poll-day-setup') against UUID columns and named a module
-- ('core-election-procedure') that no row ever created, so it could not load.
--
-- `ON CONFLICT DO NOTHING` makes a re-run a no-op.

INSERT INTO training_lms_courses (id, title, version, published_at) VALUES
('a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'Poll Worker Certification (M38)', 1, '2025-01-15 08:00:00')
ON CONFLICT (id) DO NOTHING;

INSERT INTO training_lms_modules (id, title, target_audience, module_type, pass_mark_pct) VALUES
('11111111-1111-4111-8111-111111111101', 'Poll-Day Setup & Equipment', 'PO,DPO',         'CORE', 82),
('11111111-1111-4111-8111-111111111102', 'Voter Verification',         'PO,DPO,CLERK',   'CORE', 82),
('11111111-1111-4111-8111-111111111103', 'Data Upload & Sync',         'PO,DPO',         'CORE', 82),
('11111111-1111-4111-8111-111111111104', 'Inventory Tracking',         'PO,DPO,SUPPORT', 'CORE', 82),
('11111111-1111-4111-8111-111111111105', 'Marshaller & Workforce',     'PO,DPO',         'CORE', 82),
('11111111-1111-4111-8111-111111111106', 'Closing & Reconciliation',   'PO,DPO',         'CORE', 82)
ON CONFLICT (id) DO NOTHING;

-- Lessons 1-4 are the generic induction sequence; each is drawn from a module
-- above rather than from a module that does not exist.
INSERT INTO training_lms_lessons (id, course_id, module_id, order_index, title, content_markdown, duration_minutes) VALUES
('22222222-2222-4222-8222-222222222201', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '11111111-1111-4111-8111-111111111101',  1, 'Election Procedure Overview (Text)',      'Election procedure: opening, verification, voting, tally, close. §34/DPA §6.', 20),
('22222222-2222-4222-8222-222222222202', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '11111111-1111-4111-8111-111111111102',  2, 'Voter Verification (Slides)',             'NBIS Bozorth3 → match ≥82%. Audit chain per ADR-0009.', 15),
('22222222-2222-4222-8222-222222222203', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '11111111-1111-4111-8111-111111111103',  3, 'Poll-Day Security (Audio)',               'Canary tokens, HSM attestation, freeze mode, falco #42.', 15),
('22222222-2222-4222-8222-222222222204', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '11111111-1111-4111-8111-111111111106',  4, 'Medical First Response (Video)',          'KRCS/MoH dispatch, triage, GPS, resolution.', 25),
('22222222-2222-4222-8222-222222222205', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '11111111-1111-4111-8111-111111111101',  5, 'P1: Equipment Check-In & Setup (Procedural)', 'Seal verification, device boot, network sync, biometric self-test, canary array check, HSM attestation. 30 min.', 30),
('22222222-2222-4222-8222-222222222206', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '11111111-1111-4111-8111-111111111102',  6, 'P2: Voter Verification Using Biometrics (Procedural)', 'ID check, NBIS Bozorth3 extraction, 82% threshold, decision logic, audit event emission, queue management, failure modes. 45 min.', 45),
('22222222-2222-4222-8222-222222222207', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '11111111-1111-4111-8111-111111111103',  7, 'P3: Data Upload & Election-Day Sync (Procedural)', 'Offline-first architecture, upload procedure, hash verification, scheduled sync points, failure handling. 25 min.', 25),
('22222222-2222-4222-8222-222222222208', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '11111111-1111-4111-8111-111111111104',  8, 'P4: Inventory Tracking & Asset Management (Procedural)', 'Asset scanning, condition assessment, consumable counting, opening/closing inventory, damage reporting, replacement. 20 min.', 20),
('22222222-2222-4222-8222-222222222209', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '11111111-1111-4111-8111-111111111105',  9, 'P5: Marshaller — Workforce Management (Procedural)', 'Roster verification, ID check, role assignment, roll call, shift rotation, break management, reserve pool, end-of-shift report. 35 min.', 35),
('22222222-2222-4222-8222-22222222220a', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '11111111-1111-4111-8111-111111111106', 10, 'P6: Station Closing & Reconciliation (Procedural)', 'Voter clearance, tally verification, evidence security, inventory return, workforce end report, final audit, station security. 30 min.', 30)
ON CONFLICT (id) DO NOTHING;

INSERT INTO training_lms_lesson_attempts (id, worker_id, lesson_id, completed, score_pct, idempotency_key) VALUES
('33333333-3333-4333-8333-333333333301', '44444444-4444-4444-8444-444444444401', '22222222-2222-4222-8222-222222222205', true, 88.50, 'idemp-001'),
('33333333-3333-4333-8333-333333333302', '44444444-4444-4444-8444-444444444401', '22222222-2222-4222-8222-222222222206', true, 92.00, 'idemp-002'),
('33333333-3333-4333-8333-333333333303', '44444444-4444-4444-8444-444444444401', '22222222-2222-4222-8222-222222222207', true, 87.50, 'idemp-003')
ON CONFLICT (id) DO NOTHING;

INSERT INTO training_lms_cert_audit (id, event_type, worker_id, proof_hash, correlation_key) VALUES
('55555555-5555-4555-8555-555555555501', 'CERT_ISSUED', '44444444-4444-4444-8444-444444444401', 'sha256-a1b2c3d4', 'cert-poll-worker-v1')
ON CONFLICT (id) DO NOTHING;
