-- workforce-svc V2 (design: docs/workforce-lifecycle.md §3.3, §4, §6).
-- Adds the interview record context (§3.3), worker notifications (§4/§8),
-- and the just-in-time training link on task templates (§6).

-- ---------------------------------------------------------------------------
-- S3.3: Structured interview records for shortlisted borderline candidates
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS workforce_interviews (
    interview_id UUID PRIMARY KEY,
    case_id      UUID        NOT NULL REFERENCES workforce_recruitment_cases (case_id),
    interviewer  TEXT        NOT NULL,
    scheduled_at TIMESTAMPTZ,
    score        REAL,
    notes        TEXT        NOT NULL DEFAULT '',
    outcome      TEXT        NOT NULL CHECK (outcome IN ('recommend', 'hold', 'reject')),
    recorded_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workforce_interviews_case ON workforce_interviews (case_id, recorded_at);

-- ---------------------------------------------------------------------------
-- S4/S8: Worker notifications (posting, payment, appointment, training,
-- message). Channel + delivery_ref are hashes/channel names only (DPA 2019);
-- the field-mobile push adapter resolves the recipient.
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS workforce_notifications (
    notification_id UUID PRIMARY KEY,
    worker_id       UUID        NOT NULL REFERENCES workforce_workers (worker_id),
    kind            TEXT        NOT NULL CHECK (kind IN ('posting', 'payment', 'appointment', 'training', 'message')),
    channel         TEXT        CHECK (channel IN ('phone', 'whatsapp', 'email', 'sms')),
    subject         TEXT        NOT NULL,
    body            TEXT        NOT NULL,
    delivery_ref    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workforce_notifications_worker
    ON workforce_notifications (worker_id, created_at);

-- ---------------------------------------------------------------------------
-- S6: Just-in-time training link on task templates
-- ---------------------------------------------------------------------------

ALTER TABLE workforce_task_templates
    ADD COLUMN IF NOT EXISTS training_content_id TEXT;
