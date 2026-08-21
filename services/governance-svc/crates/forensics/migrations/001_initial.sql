-- M66 election-financial-forensics schema
CREATE TABLE IF NOT EXISTS analysis_cycles (
    id              TEXT PRIMARY KEY,
    election_id     TEXT NOT NULL,
    method          TEXT NOT NULL CHECK (method IN ('RD','IV','DID')),
    status          TEXT NOT NULL DEFAULT 'Draft' CHECK (status IN ('Draft','Running','Complete','Published')),
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS estimate_records (
    id                      TEXT PRIMARY KEY,
    cycle_id                TEXT NOT NULL REFERENCES analysis_cycles(id),
    constituency_id         TEXT NOT NULL,
    estimate_value          DOUBLE PRECISION NOT NULL,
    confidence_interval_low DOUBLE PRECISION NOT NULL,
    confidence_interval_high DOUBLE PRECISION NOT NULL,
    p_value                 DOUBLE PRECISION NOT NULL,
    model_version           TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS referral_packets (
    id              TEXT PRIMARY KEY,
    cycle_id        TEXT NOT NULL REFERENCES analysis_cycles(id),
    referred_to     TEXT NOT NULL CHECK (referred_to IN ('FRC','KNBS','ODPP')),
    referral_basis  TEXT NOT NULL,
    content_url     TEXT NOT NULL,
    referred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    acknowledged_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS replication_packages (
    id                  TEXT PRIMARY KEY,
    cycle_id            TEXT NOT NULL REFERENCES analysis_cycles(id),
    code_url            TEXT NOT NULL,
    data_hash           TEXT NOT NULL,
    environment_spec    TEXT NOT NULL,
    doi                 TEXT,
    published_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS public_reports (
    id                      TEXT PRIMARY KEY,
    cycle_id                TEXT NOT NULL REFERENCES analysis_cycles(id),
    title                   TEXT NOT NULL,
    summary                 TEXT NOT NULL,
    replication_package_id  TEXT REFERENCES replication_packages(id),
    published_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_cycles_status ON analysis_cycles(status);
CREATE INDEX IF NOT EXISTS idx_cycles_election ON analysis_cycles(election_id);
CREATE INDEX IF NOT EXISTS idx_estimates_cycle ON estimate_records(cycle_id);
CREATE INDEX IF NOT EXISTS idx_referrals_cycle ON referral_packets(cycle_id);
