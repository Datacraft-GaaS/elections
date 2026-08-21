-- Adjudication cases for ambiguous match scores requiring human review.
-- Data class B per ADR-0016.
CREATE TABLE IF NOT EXISTS eel_adjudication_cases (
    case_id             TEXT        PRIMARY KEY,
    match_result_id     TEXT        NOT NULL REFERENCES eel_match_results(result_id),
    probe_ref           TEXT        NOT NULL,
    candidate_ref       TEXT        NOT NULL,
    match_score         REAL        NOT NULL,
    status              TEXT        NOT NULL DEFAULT 'open'
                                    CHECK (status IN ('open','in_review','resolved','escalated')),
    investigator_id     TEXT,
    decision            TEXT        CHECK (decision IN ('confirm_duplicate','dismiss_duplicate','escalated')),
    reason_code         TEXT,
    resolution_notes    TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at         TIMESTAMPTZ,
    sla_deadline_at     TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours'
);

CREATE INDEX IF NOT EXISTS idx_eel_adjudication_status   ON eel_adjudication_cases(status);
CREATE INDEX IF NOT EXISTS idx_eel_adjudication_created  ON eel_adjudication_cases(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_eel_adjudication_sla      ON eel_adjudication_cases(sla_deadline_at) WHERE status NOT IN ('resolved');
