-- Reviewer actions: append-only audit log per adjudication case.
-- Each row is HSM-signed; prev_action_hash forms a per-case hash chain.
-- Trigger enforces append-only: no UPDATE or DELETE allowed.

CREATE TABLE IF NOT EXISTS eel_reviewer_actions (
    id              UUID        PRIMARY KEY,
    case_id         UUID        NOT NULL REFERENCES eel_adjudication_cases_v2(id),
    reviewer_id     UUID        NOT NULL,
    action_type     TEXT        NOT NULL
                                CHECK (action_type IN (
                                    'Assign','StartReview','Decide','Escalate',
                                    'AddEvidence','Withdraw','Reopen','PanelVote'
                                )),
    comment         TEXT,
    evidence_added  JSONB       NOT NULL DEFAULT '[]',
    taken_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    hsm_signature   BYTEA       NOT NULL,
    public_key_hex  TEXT        NOT NULL,
    prev_action_hash TEXT,
    canonical_json  TEXT        NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_eel_reviewer_actions_case
    ON eel_reviewer_actions (case_id, taken_at);

-- Append-only enforcement: prevent UPDATE and DELETE on this table.
CREATE OR REPLACE FUNCTION eel_reviewer_actions_append_only()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'eel_reviewer_actions is append-only; UPDATE/DELETE are not permitted';
END;
$$;

CREATE TRIGGER trg_eel_reviewer_actions_no_update
    BEFORE UPDATE ON eel_reviewer_actions
    FOR EACH ROW EXECUTE FUNCTION eel_reviewer_actions_append_only();

CREATE TRIGGER trg_eel_reviewer_actions_no_delete
    BEFORE DELETE ON eel_reviewer_actions
    FOR EACH ROW EXECUTE FUNCTION eel_reviewer_actions_append_only();
