-- Template index: biometric template metadata only.
-- Actual encrypted templates stored by reference (probe_ref) in secure store.
-- Data class A (voter PII/biometric) per ADR-0016.
CREATE TABLE IF NOT EXISTS eel_template_index (
    index_id         TEXT        PRIMARY KEY,   -- UUIDv7 string
    voter_id         TEXT        NOT NULL,
    modality         TEXT        NOT NULL CHECK (modality IN ('fingerprint','iris','face','multimodal')),
    template_hash    TEXT        NOT NULL,       -- SHA-256 of encrypted template blob
    probe_ref        TEXT        NOT NULL,       -- URI reference to encrypted blob in secure store
    enclave_attest_id TEXT,                      -- FK to eel_enclave_attestations
    quality_score    REAL,
    enrolled_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    election_id      TEXT        NOT NULL,
    deleted_at       TIMESTAMPTZ,               -- soft-delete per ADR-0016
    CONSTRAINT chk_modality CHECK (modality IN ('fingerprint','iris','face','multimodal'))
);

CREATE INDEX IF NOT EXISTS idx_eel_template_index_voter_id    ON eel_template_index(voter_id);
CREATE INDEX IF NOT EXISTS idx_eel_template_index_modality    ON eel_template_index(modality);
CREATE INDEX IF NOT EXISTS idx_eel_template_index_election_id ON eel_template_index(election_id);
CREATE INDEX IF NOT EXISTS idx_eel_template_index_deleted_at  ON eel_template_index(deleted_at) WHERE deleted_at IS NULL;
