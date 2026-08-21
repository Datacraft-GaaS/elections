-- M40 Election Court IT Interface tables (WP-5.1)

CREATE TABLE court_petitions (
    petition_id         TEXT PRIMARY KEY,
    petitioner_hash     TEXT NOT NULL,
    contest_code        TEXT NOT NULL,
    grounds             TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'filed'
                            CHECK (status IN ('filed','served','hearing','judgment','closed')),
    filed_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    judgment_at         TIMESTAMPTZ,
    ecourt_ref          TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE court_evidence_bundles (
    bundle_id           TEXT PRIMARY KEY,
    petition_id         TEXT NOT NULL REFERENCES court_petitions(petition_id),
    bundle_version      INTEGER NOT NULL DEFAULT 1,
    item_count          INTEGER NOT NULL DEFAULT 0,
    items               JSONB NOT NULL DEFAULT '[]',
    sig                 TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE court_technical_witnesses (
    witness_id          TEXT PRIMARY KEY,
    petition_id         TEXT NOT NULL REFERENCES court_petitions(petition_id),
    name                TEXT NOT NULL,
    expertise           TEXT NOT NULL,
    organisation        TEXT NOT NULL,
    pre_registered_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ecourt_ref          TEXT
);

CREATE TABLE court_supplementary_requests (
    request_id          TEXT PRIMARY KEY,
    petition_id         TEXT NOT NULL REFERENCES court_petitions(petition_id),
    requested_by_hash   TEXT NOT NULL,
    description         TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'pending'
                            CHECK (status IN ('pending','approved','rejected','fulfilled')),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at         TIMESTAMPTZ
);

CREATE INDEX idx_petitions_status ON court_petitions(status);
CREATE INDEX idx_petitions_contest ON court_petitions(contest_code);
