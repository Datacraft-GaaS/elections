-- KEK version bindings, blob manifest, PAD (presentation-attack) events.

-- Per-cycle KEK generations issued by HSM signer.
CREATE TABLE IF NOT EXISTS eel_kek_generations (
	kek_version         INT         PRIMARY KEY,
	hsm_key_id          TEXT        NOT NULL,
	cycle_id            TEXT        NOT NULL,       -- election cycle scope
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	activated_at        TIMESTAMPTZ,
	retired_at          TIMESTAMPTZ,                -- readable but not encrypting
	rotated_by          TEXT
);

-- Which KEK generation each template blob was encrypted under.
-- Enables online re-wrap on KEK rotation without stopping ingest.
CREATE TABLE IF NOT EXISTS eel_template_kek_bindings (
	binding_id          UUID        PRIMARY KEY,
	template_id         UUID        NOT NULL REFERENCES eel_gallery_templates(template_id),
	kek_version         INT         NOT NULL REFERENCES eel_kek_generations(kek_version),
	dek_wrapped         BYTEA       NOT NULL,       -- DEK encrypted under KEK
	rewrap_pending      BOOLEAN     NOT NULL DEFAULT FALSE,
	last_verified_at    TIMESTAMPTZ,
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_template_kek_binding
	ON eel_template_kek_bindings (template_id, kek_version);
CREATE INDEX IF NOT EXISTS idx_eel_template_kek_pending_rewrap
	ON eel_template_kek_bindings (kek_version)
	WHERE rewrap_pending = TRUE;

-- Blob manifest: dangling probe_refs are prevented by tracking each blob here.
-- Every probe_ref in match_requests / gallery_templates.blob_ref MUST have a row.
CREATE TABLE IF NOT EXISTS eel_blob_manifest (
	blob_ref            TEXT        PRIMARY KEY,    -- minio://bucket/key
	sha256              BYTEA       NOT NULL,
	size_bytes          BIGINT      NOT NULL CHECK (size_bytes > 0),
	kek_version         INT         NOT NULL REFERENCES eel_kek_generations(kek_version),
	dek_wrapped         BYTEA       NOT NULL,
	minio_bucket        TEXT        NOT NULL,
	minio_object_key    TEXT        NOT NULL,
	class               TEXT        NOT NULL CHECK (class IN ('restricted','sensitive','public')),
	retention_until     TIMESTAMPTZ NOT NULL,       -- per ADR-0016
	uploaded_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	uploaded_by         TEXT,
	deleted_at          TIMESTAMPTZ,
	delete_reason       TEXT
);
CREATE INDEX IF NOT EXISTS idx_eel_blob_manifest_retention
	ON eel_blob_manifest (retention_until)
	WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_eel_blob_manifest_class
	ON eel_blob_manifest (class);

-- Presentation-attack detection events. One per attempted capture that ran PAD.
CREATE TABLE IF NOT EXISTS eel_pad_events (
	event_id            UUID        PRIMARY KEY,
	enrolment_job_id    UUID        REFERENCES eel_enrolment_jobs(id),
	kit_id              TEXT        REFERENCES eel_kits(kit_id),
	modality            TEXT        NOT NULL CHECK (modality IN ('fingerprint','face','iris')),
	is_live             BOOLEAN     NOT NULL,
	pad_score           REAL        NOT NULL CHECK (pad_score BETWEEN 0.0 AND 1.0),
	attack_type         TEXT        CHECK (attack_type IN (
	                                    'printed_photo','display_replay','silicone','latex','mask','cadaver'
	                                )),
	detector_name       TEXT        NOT NULL,
	detector_version    TEXT        NOT NULL,
	detected_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	action_taken        TEXT        NOT NULL CHECK (action_taken IN ('accepted','retake_requested','rejected','escalated'))
);
CREATE INDEX IF NOT EXISTS idx_eel_pad_events_kit_time
	ON eel_pad_events (kit_id, detected_at DESC)
	WHERE is_live = FALSE;
CREATE INDEX IF NOT EXISTS idx_eel_pad_events_attack_type
	ON eel_pad_events (attack_type, detected_at DESC)
	WHERE is_live = FALSE;
