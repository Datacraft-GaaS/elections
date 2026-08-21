-- M42+M43: Media and Observer Accreditation
-- migration: 0003_training_accreditation

-- M42: Media organisations

CREATE TABLE training_accred_media_orgs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    mck_registered  BOOLEAN NOT NULL DEFAULT false,
    status          TEXT NOT NULL DEFAULT 'pending',        -- pending, approved, suspended, revoked
    mou_signed_at   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE training_accred_media_accreditations (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organisation_id         UUID NOT NULL REFERENCES training_accred_media_orgs (id),
    mck_membership_number   TEXT NOT NULL,
    access_zones            TEXT[] NOT NULL DEFAULT '{}',
    badge_serial            TEXT UNIQUE,
    badge_issued_at         TIMESTAMPTZ,
    badge_hsm_signature     TEXT,                           -- P06 HSM signature (base64)
    status                  TEXT NOT NULL DEFAULT 'pending',-- pending, approved, badge_issued, revoked
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
    -- NPS advisory data is session-cleared and not persisted here
);

CREATE INDEX idx_accred_media_badge_serial   ON training_accred_media_accreditations (badge_serial);
CREATE INDEX idx_accred_media_org_id         ON training_accred_media_accreditations (organisation_id);
CREATE INDEX idx_accred_media_status         ON training_accred_media_accreditations (status);

CREATE TABLE training_accred_badge_revocations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    accreditation_id UUID NOT NULL REFERENCES training_accred_media_accreditations (id),
    badge_serial    TEXT NOT NULL,
    officer_id      UUID NOT NULL,
    reason          TEXT NOT NULL,
    revoked_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_accred_badge_revocations_serial ON training_accred_badge_revocations (badge_serial);

CREATE TABLE training_accred_audit_entries (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id    UUID NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id   UUID NOT NULL,
    action      TEXT NOT NULL,
    metadata    JSONB NOT NULL DEFAULT '{}',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_accred_audit_entity    ON training_accred_audit_entries (entity_type, entity_id);
CREATE INDEX idx_accred_audit_actor     ON training_accred_audit_entries (actor_id);

-- M43: Observer organisations and individuals

CREATE TABLE training_accred_observer_orgs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    nationality     TEXT NOT NULL,
    org_type        TEXT NOT NULL,                          -- domestic, international
    mou_signed_at   TIMESTAMPTZ,
    status          TEXT NOT NULL DEFAULT 'pending',        -- pending, approved, suspended
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE training_accred_observers (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organisation_id         UUID NOT NULL REFERENCES training_accred_observer_orgs (id),
    nationality             TEXT NOT NULL,
    deployment_county_codes TEXT[] NOT NULL DEFAULT '{}',
    credential_serial       TEXT UNIQUE,
    credential_issued_at    TIMESTAMPTZ,
    credential_hsm_signature TEXT,                          -- P06 HSM signature (base64)
    status                  TEXT NOT NULL DEFAULT 'pending',-- pending, approved, deployed, credential_revoked
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
    -- full_name and id_number: data class B/PII, stored in separate encrypted vault
);

CREATE INDEX idx_accred_observers_credential_serial ON training_accred_observers (credential_serial);
CREATE INDEX idx_accred_observers_org_id            ON training_accred_observers (organisation_id);
CREATE INDEX idx_accred_observers_status            ON training_accred_observers (status);

CREATE TABLE training_accred_credential_revocations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    observer_id     UUID NOT NULL REFERENCES training_accred_observers (id),
    credential_serial TEXT NOT NULL,
    officer_id      UUID NOT NULL,
    reason          TEXT NOT NULL,
    revoked_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_accred_cred_revocations_serial ON training_accred_credential_revocations (credential_serial);
