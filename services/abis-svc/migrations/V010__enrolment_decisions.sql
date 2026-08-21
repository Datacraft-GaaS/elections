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
