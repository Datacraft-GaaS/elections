-- Adjudication cases: one per ambiguous dedup result requiring human review.
-- Supersedes V004 (legacy schema); this table is owned by eel-abis-adjudication.
-- managed=False in Django; Rust migrations own the schema.

CREATE TABLE IF NOT EXISTS eel_adjudication_cases_v2 (
    id                          UUID        PRIMARY KEY,
    opened_at                   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    priority                    TEXT        NOT NULL DEFAULT 'Standard'
                                            CHECK (priority IN ('Standard','Urgent','ElectionDayHold')),
    tier                        TEXT        NOT NULL DEFAULT 'Tier1Analyst'
                                            CHECK (tier IN ('Tier1Analyst','Tier2Supervisor','Tier3Panel')),
    state                       TEXT        NOT NULL DEFAULT 'Open'
                                            CHECK (state IN ('Open','Assigned','InReview','Decided','Escalated','Withdrawn','Reopened')),
    enrolment_job_id            UUID        REFERENCES eel_enrolment_jobs(id),
    probable_match_subject_ids  JSONB       NOT NULL DEFAULT '[]',
    evidence_bundle_ref         UUID        NOT NULL,
    assigned_to                 UUID,
    assigned_at                 TIMESTAMPTZ,
    decided_at                  TIMESTAMPTZ,
    decision_type               TEXT        CHECK (decision_type IN (
                                    'ConfirmMatch','ConfirmNoMatch','MergeInto',
                                    'SplitFrom','Withdraw','Escalate'
                                )),
    decision_subject_id         UUID,
    decision_reason             TEXT,
    sla_deadline_at             TIMESTAMPTZ NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_eel_adj_cases_v2_state_priority
    ON eel_adjudication_cases_v2 (state, priority DESC, sla_deadline_at)
    WHERE state NOT IN ('Decided','Withdrawn');

CREATE INDEX IF NOT EXISTS idx_eel_adj_cases_v2_assigned
    ON eel_adjudication_cases_v2 (assigned_to)
    WHERE state IN ('Assigned','InReview');
