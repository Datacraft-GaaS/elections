-- Card blacklist for lost/stolen voter ID cards (M53, WP-4.10).
-- Propagation to BVI edge nodes happens via the transactional outbox
-- (topic eel.bvr.card_blacklisted.v1); the row itself is the source of truth.

CREATE TABLE IF NOT EXISTS card_blacklist (
    id             UUID        NOT NULL PRIMARY KEY,
    card_number    TEXT        NOT NULL UNIQUE,
    reason         TEXT        NOT NULL,
    blacklisted_at TIMESTAMPTZ NOT NULL,
    source         TEXT        NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_card_blacklist_number ON card_blacklist (card_number);
