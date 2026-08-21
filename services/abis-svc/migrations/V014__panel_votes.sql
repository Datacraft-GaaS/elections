-- Panel votes for Tier3 cases requiring M-of-N approval.
-- One row per reviewer per case; unique constraint prevents double-voting.

CREATE TABLE IF NOT EXISTS eel_panel_votes (
    id          UUID        PRIMARY KEY,
    case_id     UUID        NOT NULL REFERENCES eel_adjudication_cases_v2(id),
    reviewer_id UUID        NOT NULL,
    vote        TEXT        NOT NULL CHECK (vote IN ('Approve','Reject','Abstain')),
    rationale   TEXT,
    voted_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    hsm_signature BYTEA     NOT NULL,
    public_key_hex TEXT     NOT NULL
);

-- Each reviewer can only vote once per case.
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_panel_votes_reviewer_case
    ON eel_panel_votes (case_id, reviewer_id);

CREATE INDEX IF NOT EXISTS idx_eel_panel_votes_case
    ON eel_panel_votes (case_id, voted_at);
