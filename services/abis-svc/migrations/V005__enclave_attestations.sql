-- Enclave attestation records: TDX/SEV-SNP or software profile.
-- Data class C per ADR-0016.
CREATE TABLE IF NOT EXISTS eel_enclave_attestations (
    enclave_id       TEXT        PRIMARY KEY,
    worker_id        TEXT        NOT NULL,
    profile          TEXT        NOT NULL CHECK (profile IN ('software','tdx','sev_snp')),
    attestation_hash TEXT        NOT NULL,
    batch_id         TEXT,
    verified_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_eel_enclave_attestations_worker ON eel_enclave_attestations(worker_id);
CREATE INDEX IF NOT EXISTS idx_eel_enclave_attestations_batch  ON eel_enclave_attestations(batch_id);
