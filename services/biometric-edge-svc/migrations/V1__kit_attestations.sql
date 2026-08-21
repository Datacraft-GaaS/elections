-- Migration V1: kit_attestations table (WP-3.8)
-- Stores last-seen attestation per kit for revocation-check consumers.
CREATE TABLE kit_attestations (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_serial    TEXT NOT NULL,
    attestation_kind TEXT NOT NULL CHECK (attestation_kind IN ('play_integrity', 'app_attest', 'tpm_quote', 'legacy_software')),
    integrity_level  TEXT NOT NULL CHECK (integrity_level IN ('MEETS_STRONG', 'MEETS_DEVICE', 'MEETS_BASIC')),
    kit_tier         TEXT NOT NULL CHECK (kit_tier IN ('play_integrity', 'app_attest', 'tpm', 'legacy')),
    cert_thumbprint  TEXT NOT NULL,
    legacy_warning   BOOLEAN NOT NULL DEFAULT FALSE,
    attested_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at       TIMESTAMPTZ NOT NULL,
    revoked          BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at       TIMESTAMPTZ,
    revoked_reason   TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX kit_attestations_device_serial_latest
    ON kit_attestations (device_serial, attested_at DESC);

CREATE INDEX kit_attestations_expires_at ON kit_attestations (expires_at)
    WHERE revoked = FALSE;

COMMENT ON TABLE kit_attestations IS 'Last-seen hardware/software attestation record per kit device (WP-3.8).';
