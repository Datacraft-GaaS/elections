-- G23 — Section 44A Complementary Mechanism Drill (CLOSED)
-- Migration creates procedural lesson P7: Escalation Protocol & Complementary Mechanism
-- References: Elections Act §44A, ADR-0019 (security), ADR-0023 (audit)
-- Lesson P7 covers: escalation ladder, manual form collection, rapid-response activation, CCB convening
INSERT INTO lms_app_lesson (id, module_id, title, order_in_module, audit_ref, proof_hash, created_at)
VALUES (7, 6, 'Section 44A Escalation & Complementary Mechanism', 7,
        'g23-section44a-lesson-p7',
        'sha256(section44a-procedural-lesson-v2)',
        NOW());
