-- eel transparency log schema (WP-2.1, ADR-0017)

CREATE SEQUENCE IF NOT EXISTS leaves_index_seq START 0 INCREMENT 1 MINVALUE 0;

CREATE TABLE IF NOT EXISTS leaves (
    id                  UUID        NOT NULL DEFAULT gen_random_uuid(),
    index               BIGINT      NOT NULL DEFAULT nextval('leaves_index_seq'),
    domain              TEXT        NOT NULL,
    hash                BYTEA       NOT NULL,
    producer_spiffe_id  TEXT        NOT NULL,
    source_event_id     TEXT        NOT NULL,
    source_topic        TEXT        NOT NULL,
    event_time          TIMESTAMPTZ NOT NULL,
    appended_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    producer_signature  BYTEA       NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (index),
    UNIQUE (source_topic, source_event_id)
);

CREATE TABLE IF NOT EXISTS tree_tiles (
    level       INT         NOT NULL,
    tile_index  BIGINT      NOT NULL,
    hash        BYTEA       NOT NULL,
    computed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (level, tile_index)
);

CREATE TABLE IF NOT EXISTS sth (
    id              UUID        NOT NULL DEFAULT gen_random_uuid(),
    tree_size       BIGINT      NOT NULL,
    root_hash       BYTEA       NOT NULL,
    domain          TEXT,
    cadence         TEXT        NOT NULL,
    signed_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    signature       BYTEA       NOT NULL,
    key_role        TEXT        NOT NULL DEFAULT 'sth',
    key_fingerprint TEXT        NOT NULL,
    PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS sth_tree_size_idx ON sth (tree_size DESC);

CREATE TABLE IF NOT EXISTS sth_batch (
    id              UUID    NOT NULL DEFAULT gen_random_uuid(),
    parent_sth_id   UUID    NOT NULL REFERENCES sth(id),
    child_sth_id    UUID    NOT NULL REFERENCES sth(id),
    PRIMARY KEY (id),
    UNIQUE (parent_sth_id, child_sth_id)
);

CREATE TABLE IF NOT EXISTS witnesses (
    witness_id          TEXT        NOT NULL,
    witness_spiffe_id   TEXT        NOT NULL,
    public_key_hex      TEXT        NOT NULL,
    registered_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_internal         BOOLEAN     NOT NULL DEFAULT FALSE,
    PRIMARY KEY (witness_id)
);

CREATE TABLE IF NOT EXISTS witness_cosigs (
    id                  UUID        NOT NULL DEFAULT gen_random_uuid(),
    sth_id              UUID        NOT NULL REFERENCES sth(id),
    witness_id          TEXT        NOT NULL,
    witness_spiffe_id   TEXT        NOT NULL,
    signature           BYTEA       NOT NULL,
    cosigned_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_internal         BOOLEAN     NOT NULL DEFAULT FALSE,
    PRIMARY KEY (id),
    UNIQUE (sth_id, witness_id)
);

CREATE TABLE IF NOT EXISTS ingest_offsets (
    topic               TEXT    NOT NULL,
    partition           INT     NOT NULL,
    committed_offset    BIGINT  NOT NULL,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (topic, partition)
);

CREATE TABLE IF NOT EXISTS domains (
    name            TEXT        NOT NULL,
    description     TEXT        NOT NULL DEFAULT '',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (name)
);

INSERT INTO domains (name, description) VALUES
    ('voter-events',      'Voter registration events (M02/M03)'),
    ('results-events',    'Results capture and tallying (M08/M09/M10)'),
    ('custody-events',    'Chain of custody for election kit (M20)'),
    ('credential-events', 'Credential issuance'),
    ('meta-events',       'Meta: schema migrations, HSM ceremony ops, STH chains')
ON CONFLICT DO NOTHING;
