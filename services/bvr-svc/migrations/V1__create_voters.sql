-- BVR voter register table (T1-voter cluster)
-- Partitioned by county_code (47 counties).
-- Per ADR-0012/0013: state table + audit shadow.

CREATE TABLE IF NOT EXISTS voters (
    voter_id          TEXT        NOT NULL,
    version           BIGINT      NOT NULL DEFAULT 1,
    county_code       CHAR(3)     NOT NULL,
    -- Tier A PII stored as field-encrypted blobs (eel-crypto-rs KEK)
    national_id_enc   BYTEA       NOT NULL,
    full_name_enc     BYTEA       NOT NULL,
    dob_enc           BYTEA       NOT NULL,
    phone_enc         BYTEA,
    -- Tier B operational
    polling_station_id TEXT       NOT NULL,
    election_scope    TEXT        NOT NULL,
    status            TEXT        NOT NULL DEFAULT 'pending',
    biometric_template_refs JSONB NOT NULL DEFAULT '[]',
    is_canary         BOOLEAN     NOT NULL DEFAULT FALSE,
    key_id            TEXT        NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (voter_id, county_code)
) PARTITION BY LIST (county_code);

-- Create partitions for all 47 Kenya counties (001-047)
DO $$
DECLARE
    i INT;
    county TEXT;
BEGIN
    FOR i IN 1..47 LOOP
        county := LPAD(i::TEXT, 3, '0');
        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS voters_%s PARTITION OF voters FOR VALUES IN (%L)',
            county, county
        );
    END LOOP;
END $$;

-- Indexes
CREATE INDEX IF NOT EXISTS idx_voters_status       ON voters (status) WHERE status != 'deregistered';
CREATE INDEX IF NOT EXISTS idx_voters_election     ON voters (election_scope, county_code);
CREATE INDEX IF NOT EXISTS idx_voters_is_canary    ON voters (is_canary) WHERE is_canary = TRUE;
CREATE INDEX IF NOT EXISTS idx_voters_updated      ON voters (updated_at DESC);
