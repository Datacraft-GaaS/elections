-- Voter card API surface — postgres for public contract (no in-memory)
-- Replaces Arc<Mutex<HashMap<String, VoterCardRequest>>> in voter-card/routes/spec.rs SpecState.

CREATE TABLE IF NOT EXISTS voter_card_api_requests (
    request_id                      TEXT PRIMARY KEY,
    voter_id_hash                   TEXT NOT NULL,
    request_type                    TEXT NOT NULL,
    reason                          TEXT NOT NULL,
    preferred_collection_centre     TEXT NOT NULL,
    status                          TEXT NOT NULL DEFAULT 'submitted' CHECK (status IN ('submitted','approved','rejected','printed','collected')),
    submitted_at                    TIMESTAMPTZ NOT NULL DEFAULT now(),
    collected_at                    TIMESTAMPTZ,
    collection_centre_code          TEXT,
    officer_hash                    TEXT
);

CREATE TABLE IF NOT EXISTS voter_card_api_centres (
    centre_code     TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    county_code     TEXT NOT NULL,
    address         TEXT NOT NULL,
    operating_hours TEXT,
    open            BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX IF NOT EXISTS idx_voter_card_requests_status ON voter_card_api_requests(status);
CREATE INDEX IF NOT EXISTS idx_voter_card_requests_submitted ON voter_card_api_requests(submitted_at);

-- Reversible: DROP TABLE IF EXISTS voter_card_api_centres, voter_card_api_requests CASCADE;
