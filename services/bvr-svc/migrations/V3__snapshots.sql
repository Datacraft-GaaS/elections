-- Register snapshots and Merkle roots
CREATE TABLE IF NOT EXISTS register_snapshots (
    snapshot_id     TEXT        NOT NULL PRIMARY KEY,
    election_scope  TEXT        NOT NULL,
    station_count   INTEGER     NOT NULL,
    voter_count     BIGINT      NOT NULL,
    merkle_root     TEXT        NOT NULL,
    signed_by       TEXT        NOT NULL,
    checkpoint      TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS merkle_roots (
    id              TEXT        NOT NULL PRIMARY KEY,
    root_hash       TEXT        NOT NULL,
    seq             BIGINT      NOT NULL,
    election_scope  TEXT        NOT NULL,
    signed_by_hsm   TEXT,
    escrow_confirmed BOOLEAN    NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_merkle_seq ON merkle_roots (election_scope, seq);
