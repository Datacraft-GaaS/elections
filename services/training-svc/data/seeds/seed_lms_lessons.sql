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

INSERT INTO training_lms_courses (id, title, version, published_at) VALUES ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'Poll Worker Certification (M38)', 1, '2025-01-15 08:00:00');
INSERT INTO training_lms_modules (id, title, target_audience, language, module_type, pass_mark_pct, version) VALUES
('poll-day-setup', 'Poll-Day Setup & Equipment', 'PO,DPO', 'en', 'CORE', 82, 1),
('voter-verification', 'Voter Verification', 'PO,DPO,CLERK', 'en', 'CORE', 82, 1),
('poll-day-sync', 'Data Upload & Sync', 'PO,DPO', 'en', 'CORE', 82, 1),
('inventory-tracking', 'Inventory Tracking', 'PO,DPO,SUPPORT', 'en', 'CORE', 82, 1),
('marshaller-station-management', 'Marshaller & Workforce', 'PO,DPO', 'en', 'CORE', 82, 1),
('station-management', 'Closing & Reconciliation', 'PO,DPO', 'en', 'CORE', 82, 1) ON CONFLICT DO NOTHING;
INSERT INTO training_lms_lessons (id, course_id, module_id, order_index, title, content_markdown, duration_minutes) VALUES 
('l01-text', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'core-election-procedure', 1, 'Election Procedure Overview (Text)', 'Election procedure: opening, verification, voting, tally, close. §34/DPA §6.', 20),
('l02-slides', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'core-election-procedure', 2, 'Voter Verification (Slides)', 'NBIS Bozorth3 → match ≥82%. Audit chain per ADR-0009.', 15),
('l03-audio', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'core-election-procedure', 3, 'Poll-Day Security (Audio)', 'Canary tokens, HSM attestation, freeze mode, falco #42.', 15),
('l04-video', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'core-election-procedure', 4, 'Medical First Response (Video)', 'KRCS/MoH dispatch, triage, GPS, resolution.', 25),
('P1-equip', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'poll-day-setup', 5, 'P1: Equipment Check-In & Setup (Procedural)', 'Seal verification, device boot, network sync, biometric self-test, canary array check, HSM attestation. 30 min.', 30),
('P2-verify', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'voter-verification', 6, 'P2: Voter Verification Using Biometrics (Procedural)', 'ID check, NBIS Bozorth3 extraction, 82% threshold, decision logic, audit event emission, queue management, failure modes. 45 min.', 45),
('P3-sync', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'poll-day-sync', 7, 'P3: Data Upload & Election-Day Sync (Procedural)', 'Offline-first architecture, upload procedure, hash verification, scheduled sync points, failure handling. 25 min.', 25),
('P4-inv', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'inventory-tracking', 8, 'P4: Inventory Tracking & Asset Management (Procedural)', 'Asset scanning, condition assessment, consumable counting, opening/closing inventory, damage reporting, replacement. 20 min.', 20),
('P5-marsh', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'marshaller-station-management', 9, 'P5: Marshaller — Workforce Management (Procedural)', 'Roster verification, ID check, role assignment, roll call, shift rotation, break management, reserve pool, end-of-shift report. 35 min.', 35),
('P6-close', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'station-management', 10, 'P6: Station Closing & Reconciliation (Procedural)', 'Voter clearance, tally verification, evidence security, inventory return, workforce end report, final audit, station security. 30 min.', 30);
INSERT INTO training_lms_lesson_attempts (id, worker_id, lesson_id, completed, score_pct, idempotency_key) VALUES
('att-01-test', 'w-001', 'P1-equip', true, 88.50, 'idemp-001'),
('att-02-verify', 'w-001', 'P2-verify', true, 92.00, 'idemp-002'),
('att-03-sync', 'w-001', 'P3-sync', true, 87.50, 'idemp-003');
INSERT INTO training_lms_cert_audit (id, event_type, worker_id, proof_hash, correlation_key) VALUES
('audit-cert-001', 'CERT_ISSUED', 'w-001', 'sha256-a1b2c3d4', 'cert-poll-worker-v1');
