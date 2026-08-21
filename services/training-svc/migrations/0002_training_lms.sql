-- M38: Poll Worker Training LMS
-- migration: 0002_training_lms

CREATE TABLE training_lms_modules (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title           TEXT NOT NULL,
    target_audience TEXT NOT NULL,
    module_type     TEXT NOT NULL,                          -- core, refresher, briefing
    pass_mark_pct   SMALLINT NOT NULL CHECK (pass_mark_pct BETWEEN 0 AND 100),
    content_url     TEXT,
    published_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE training_lms_enrollments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    worker_id       UUID NOT NULL,
    module_id       UUID NOT NULL REFERENCES training_lms_modules (id),
    status          TEXT NOT NULL DEFAULT 'enrolled',       -- enrolled, in_progress, passed, failed, expired
    attempt_count   SMALLINT NOT NULL DEFAULT 0,
    last_score_pct  SMALLINT,
    enrolled_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at    TIMESTAMPTZ,
    UNIQUE (worker_id, module_id)
);

CREATE INDEX idx_lms_enrollments_worker_id  ON training_lms_enrollments (worker_id);
CREATE INDEX idx_lms_enrollments_status     ON training_lms_enrollments (status);
CREATE INDEX idx_lms_enrollments_module_id  ON training_lms_enrollments (module_id);

CREATE TABLE training_lms_certificates (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    worker_id           UUID NOT NULL,
    certificate_hash    TEXT NOT NULL,                      -- SHA-256 of certificate PDF
    qr_verification_url TEXT NOT NULL,
    hsm_signature       TEXT NOT NULL,                      -- P06 HSM signature (base64)
    issued_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at          TIMESTAMPTZ NOT NULL,
    revoked             BOOLEAN NOT NULL DEFAULT false,
    revoked_at          TIMESTAMPTZ,
    revoked_reason      TEXT
);

CREATE INDEX idx_lms_certificates_worker_id ON training_lms_certificates (worker_id);
CREATE INDEX idx_lms_certificates_revoked   ON training_lms_certificates (revoked);

CREATE TABLE training_lms_progress (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scope                   TEXT NOT NULL,                  -- national, county, constituency
    scope_code              TEXT,
    total_workers           INTEGER NOT NULL DEFAULT 0,
    certified_count         INTEGER NOT NULL DEFAULT 0,
    refresher_pending_count INTEGER NOT NULL DEFAULT 0,
    not_certified_count     INTEGER NOT NULL DEFAULT 0,
    snapshot_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_lms_progress_scope      ON training_lms_progress (scope, scope_code);
CREATE INDEX idx_lms_progress_snapshot   ON training_lms_progress (snapshot_at);
