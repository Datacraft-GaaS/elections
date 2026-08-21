-- Enrolment jobs: one row per biometric capture event submitted for 1:N dedup.
-- UUID7 PKs (monotonic, sortable). JSONB for quality_report flexibility.
-- Orchestrator uses (status, created_at) index with SELECT FOR UPDATE SKIP LOCKED.

CREATE TABLE IF NOT EXISTS eel_enrolment_jobs (
    id                  UUID        PRIMARY KEY,
    subject_id          UUID        NOT NULL,
    source              TEXT        NOT NULL
                                    CHECK (source IN ('BvrKit','LegacyImport','Diaspora','Rectification')),
    status              TEXT        NOT NULL DEFAULT 'Received'
                                    CHECK (status IN (
                                        'Received','QualityChecking','DedupPending',
                                        'DedupInProgress','DedupComplete','DedupHold',
                                        'AdjudicationRequired','Enrolled','Rejected','Withdrawn'
                                    )),
    captured_at         TIMESTAMPTZ NOT NULL,
    captured_by_kit_id  TEXT,
    captured_by_officer TEXT,
    quality_report      JSONB,
    templates           JSONB        NOT NULL DEFAULT '[]',
    lease_expires_at    TIMESTAMPTZ,
    worker_id           TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_eel_enrolment_jobs_status_created
    ON eel_enrolment_jobs (status, created_at)
    WHERE status NOT IN ('Enrolled','Rejected','Withdrawn');

CREATE INDEX IF NOT EXISTS idx_eel_enrolment_jobs_subject
    ON eel_enrolment_jobs (subject_id);

CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_enrolment_jobs_subject_source_captured
    ON eel_enrolment_jobs (subject_id, source, captured_at);
