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

-- Persisted fairness evaluations (trending over time) + legacy KIEMS
-- migration mapping table.

CREATE TABLE IF NOT EXISTS eel_fairness_evaluations (
	evaluation_id       UUID        PRIMARY KEY,
	evaluated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	window_from         TIMESTAMPTZ NOT NULL,
	window_to           TIMESTAMPTZ NOT NULL,
	dimension           TEXT        NOT NULL CHECK (dimension IN ('sex','age_bucket','county','disability','multimodal')),
	slice_key           TEXT        NOT NULL,
	fmr                 REAL        NOT NULL CHECK (fmr >= 0.0 AND fmr <= 1.0),
	fnmr                REAL        NOT NULL CHECK (fnmr >= 0.0 AND fnmr <= 1.0),
	sample_size         BIGINT      NOT NULL CHECK (sample_size >= 0),
	baseline_fmr        REAL,
	baseline_fnmr       REAL,
	deviation_pct       REAL,                          -- absolute deviation vs baseline
	breach              BOOLEAN     NOT NULL DEFAULT FALSE,
	breach_severity     TEXT        CHECK (breach_severity IN ('info','warn','critical')),
	model_card_version  TEXT        NOT NULL,
	notes               TEXT
);
CREATE INDEX IF NOT EXISTS idx_eel_fairness_dim_slice_time
	ON eel_fairness_evaluations (dimension, slice_key, evaluated_at DESC);
CREATE INDEX IF NOT EXISTS idx_eel_fairness_breach
	ON eel_fairness_evaluations (evaluated_at DESC)
	WHERE breach = TRUE;

-- Legacy KIEMS to eel subject mapping (WP-5.10 data-migration).
CREATE TABLE IF NOT EXISTS eel_legacy_kiems_migration (
	legacy_id           TEXT        PRIMARY KEY,       -- legacy voter id
	new_subject_id      UUID        NOT NULL,
	migrated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	source_batch_id     TEXT        NOT NULL,
	legacy_template_hash BYTEA      NOT NULL,          -- SHA-256 of legacy template
	verification_status TEXT        NOT NULL CHECK (verification_status IN ('verified','pending','failed','skipped')),
	verified_at         TIMESTAMPTZ,
	discrepancy_notes   TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_legacy_new_subject
	ON eel_legacy_kiems_migration (new_subject_id);
CREATE INDEX IF NOT EXISTS idx_eel_legacy_batch
	ON eel_legacy_kiems_migration (source_batch_id);
CREATE INDEX IF NOT EXISTS idx_eel_legacy_status
	ON eel_legacy_kiems_migration (verification_status)
	WHERE verification_status <> 'verified';

-- Model card versions used by both the matcher and fairness evaluations.
CREATE TABLE IF NOT EXISTS eel_model_cards (
	model_card_version  TEXT        PRIMARY KEY,
	modality            TEXT        NOT NULL,
	vendor_id           TEXT        REFERENCES eel_vendor_registry(vendor_id),
	fmr_target          REAL        NOT NULL,
	fnmr_target         REAL        NOT NULL,
	trained_on          TEXT,
	published_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	cosign_signature    BYTEA       NOT NULL,
	cosign_pubkey_ref   TEXT        NOT NULL,
	retired_at          TIMESTAMPTZ
);
