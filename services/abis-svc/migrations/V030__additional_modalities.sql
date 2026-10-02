-- Copyright (c) 2026 Nyimbi Odero
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.

-- Additional biometric modalities (palm, latent, voice, vein, signature)
-- matching the world-class commercial ABIS scope.
-- Modality is added to the CHECK constraint on eel_gallery_templates
-- and eel_pad_events; new template blobs stored inline for hot access.

ALTER TABLE eel_gallery_templates
	DROP CONSTRAINT IF EXISTS eel_gallery_templates_modality_check;
ALTER TABLE eel_gallery_templates
	ADD CONSTRAINT eel_gallery_templates_modality_check
	CHECK (modality IN (
		'fingerprint','face','iris','palm','latent','voice','vein_finger','vein_palm','signature','periocular','ear','gait'
	));

ALTER TABLE eel_pad_events
	DROP CONSTRAINT IF EXISTS eel_pad_events_modality_check;
ALTER TABLE eel_pad_events
	ADD CONSTRAINT eel_pad_events_modality_check
	CHECK (modality IN (
		'fingerprint','face','iris','palm','latent','voice','vein_finger','vein_palm','signature','periocular','ear','gait'
	));

-- Palm templates: 10 palms per subject (5 zones per palm) with
-- creases + minutiae + orientation field.
CREATE TABLE IF NOT EXISTS eel_palm_templates (
	template_id       UUID        PRIMARY KEY REFERENCES eel_gallery_templates(template_id),
	palm_position     TEXT        NOT NULL CHECK (palm_position IN ('right_full','left_full','right_writer','left_writer','right_lower','left_lower','right_upper','left_upper')),
	minutiae_count    INT         NOT NULL,
	crease_pattern    BYTEA       NOT NULL,
	orientation_field BYTEA,
	nfiq2_palm        REAL,
	captured_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Latent prints: forensic partials with lift metadata, distortion, and chain
-- of custody.
CREATE TABLE IF NOT EXISTS eel_latent_templates (
	template_id       UUID        PRIMARY KEY REFERENCES eel_gallery_templates(template_id),
	source            TEXT        NOT NULL CHECK (source IN ('crime_scene','lifted','powdered','chemical','digital')),
	lift_id           TEXT,
	case_ref          TEXT,
	distortion_score  REAL        NOT NULL CHECK (distortion_score BETWEEN 0.0 AND 1.0),
	usable_ridge_pct  REAL        NOT NULL,
	examiner_id       UUID,
	chain_of_custody  JSONB       NOT NULL DEFAULT '[]',
	captured_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_eel_latent_case ON eel_latent_templates (case_ref);

-- Voice / speaker verification.
CREATE TABLE IF NOT EXISTS eel_voice_templates (
	template_id       UUID        PRIMARY KEY REFERENCES eel_gallery_templates(template_id),
	utterance_type    TEXT        NOT NULL CHECK (utterance_type IN ('passphrase','free_speech','digits','text_dependent','text_independent')),
	sample_rate_hz    INT         NOT NULL,
	duration_ms       INT         NOT NULL,
	language          TEXT        NOT NULL,
	embedding_dim     INT         NOT NULL,
	embedding         BYTEA       NOT NULL,
	snr_db            REAL,
	captured_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Vein pattern (finger + palm).
CREATE TABLE IF NOT EXISTS eel_vein_templates (
	template_id       UUID        PRIMARY KEY REFERENCES eel_gallery_templates(template_id),
	vein_type         TEXT        NOT NULL CHECK (vein_type IN ('finger_vein','palm_vein','wrist_vein')),
	body_side         CHAR(1)     NOT NULL CHECK (body_side IN ('R','L')),
	nir_wavelength_nm INT         NOT NULL,
	pattern_bitmap    BYTEA       NOT NULL,
	captured_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Signature dynamics (pen tablet or on-screen).
CREATE TABLE IF NOT EXISTS eel_signature_templates (
	template_id       UUID        PRIMARY KEY REFERENCES eel_gallery_templates(template_id),
	capture_device    TEXT,
	pressure_series   BYTEA,          -- time series of pressure samples
	velocity_series   BYTEA,
	acceleration_ser  BYTEA,
	pen_up_events     INT         NOT NULL DEFAULT 0,
	captured_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Modality-aware watchlists (small denylist for fast pre-match).
CREATE TABLE IF NOT EXISTS eel_watchlists (
	watchlist_id      UUID        PRIMARY KEY,
	name              TEXT        NOT NULL,
	description       TEXT,
	modalities        TEXT[]      NOT NULL,
	max_size          INT         NOT NULL DEFAULT 10000,
	current_size      INT         NOT NULL DEFAULT 0,
	classification    TEXT        NOT NULL CHECK (classification IN ('exclusion','duplicate_alert','vip_flag','fraud_hold')),
	owner             TEXT        NOT NULL,
	created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	last_reviewed_at  TIMESTAMPTZ,
	suspended_at      TIMESTAMPTZ
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_watchlists_name ON eel_watchlists (name);

CREATE TABLE IF NOT EXISTS eel_watchlist_entries (
	entry_id          UUID        PRIMARY KEY,
	watchlist_id      UUID        NOT NULL REFERENCES eel_watchlists(watchlist_id),
	subject_id        UUID,
	template_id       UUID        REFERENCES eel_gallery_templates(template_id),
	added_by          UUID        NOT NULL,
	added_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	reason            TEXT        NOT NULL,
	sunset_at         TIMESTAMPTZ,
	last_hit_at       TIMESTAMPTZ,
	hit_count         INT         NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_eel_watchlist_entries_wl ON eel_watchlist_entries (watchlist_id);
CREATE INDEX IF NOT EXISTS idx_eel_watchlist_entries_subject ON eel_watchlist_entries (subject_id);
