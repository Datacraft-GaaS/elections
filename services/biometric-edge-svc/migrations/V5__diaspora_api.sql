-- Diaspora API surface — postgres for public contract (no in-memory)
-- Replaces Mutex<HashMap<String, Value>> embassies/ballot_dispatches/return_dispatches in diaspora/api.rs

CREATE TABLE IF NOT EXISTS diaspora_api_embassies (
    embassy_code    TEXT PRIMARY KEY,
    data            JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS diaspora_api_ballot_dispatches (
    dispatch_id     TEXT PRIMARY KEY,
    data            JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS diaspora_api_return_dispatches (
    return_id       TEXT PRIMARY KEY,
    data            JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_diaspora_embassies_code ON diaspora_api_embassies(embassy_code);
CREATE INDEX IF NOT EXISTS idx_diaspora_ballot_dispatches_id ON diaspora_api_ballot_dispatches(dispatch_id);

-- Reversible: DROP TABLE IF EXISTS diaspora_api_return_dispatches, diaspora_api_ballot_dispatches, diaspora_api_embassies CASCADE;
