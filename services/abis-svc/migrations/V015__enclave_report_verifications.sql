-- V015: Enclave report verification log.
--
-- Records every TDX / SEV-SNP / software attestation verification attempt,
-- linking it to the originating enclave worker and the nonce consumed.
-- Supports audit queries: which enclaves were attested, when, with what outcome.

CREATE TABLE IF NOT EXISTS enclave_report_verifications (
    id              UUID        NOT NULL DEFAULT gen_random_uuid(),
    enclave_id      TEXT        NOT NULL,
    worker_id       TEXT        NOT NULL,
    profile         TEXT        NOT NULL
        CHECK (profile IN ('tdx', 'sev_snp', 'software')),
    nonce_hex       TEXT        NOT NULL,
    attestation_hash TEXT       NOT NULL,
    verdict         TEXT        NOT NULL
        CHECK (verdict IN ('approved', 'rejected')),
    rejection_reason TEXT,
    mrtd_hex        TEXT,       -- TDX: MRTD hash (hex)
    measurement_hex TEXT,       -- SEV-SNP: measurement hash (hex)
    cpu_svn         SMALLINT,   -- TDX: CPU SVN
    batch_id        TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT enclave_report_verifications_pkey PRIMARY KEY (id)
);

-- Fast lookup by enclave and time for audit reports.
CREATE INDEX IF NOT EXISTS idx_erv_enclave_created
    ON enclave_report_verifications (enclave_id, created_at DESC);

-- Nonce uniqueness: each nonce may appear at most once (replay prevention
-- at the DB layer complements the in-memory NonceStore).
CREATE UNIQUE INDEX IF NOT EXISTS idx_erv_nonce_unique
    ON enclave_report_verifications (nonce_hex);

COMMENT ON TABLE enclave_report_verifications IS
    'Immutable audit log of TDX / SEV-SNP / software attestation verifications.';
