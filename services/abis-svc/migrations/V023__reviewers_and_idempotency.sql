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

-- Reviewer registry + reviewer specialisations + idempotency store.
-- Reviewers are the adjudicators for ambiguous cases per M03/M23.

CREATE TABLE IF NOT EXISTS eel_reviewers (
	reviewer_id         UUID        PRIMARY KEY,
	full_name           TEXT        NOT NULL,           -- data class Sensitive
	staff_number        TEXT        NOT NULL,
	email_hash          BYTEA       NOT NULL,           -- SHA-256 of email; raw stored in HR
	tier                TEXT        NOT NULL CHECK (tier IN ('tier1_analyst','tier2_supervisor','tier3_panel')),
	spiffe_id           TEXT        NOT NULL,
	hardware_key_id     TEXT,                           -- WebAuthn / YubiKey for high-tier
	max_daily_cases     INT         NOT NULL DEFAULT 50,
	cases_today         INT         NOT NULL DEFAULT 0,
	is_on_shift         BOOLEAN     NOT NULL DEFAULT FALSE,
	shift_started_at    TIMESTAMPTZ,
	certified_at        TIMESTAMPTZ NOT NULL,           -- M38 LMS training gate
	certification_ref   TEXT        NOT NULL,           -- LMS credential id
	suspended_at        TIMESTAMPTZ,
	suspension_reason   TEXT,
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_reviewers_staff_number
	ON eel_reviewers (staff_number);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_reviewers_spiffe_id
	ON eel_reviewers (spiffe_id);
CREATE INDEX IF NOT EXISTS idx_eel_reviewers_on_shift
	ON eel_reviewers (tier, cases_today)
	WHERE is_on_shift = TRUE AND suspended_at IS NULL;

-- Which modalities a reviewer is qualified to adjudicate.
CREATE TABLE IF NOT EXISTS eel_reviewer_specialisations (
	reviewer_id         UUID        NOT NULL REFERENCES eel_reviewers(reviewer_id),
	modality            TEXT        NOT NULL CHECK (modality IN ('fingerprint','face','iris','multimodal')),
	proficiency         SMALLINT    NOT NULL CHECK (proficiency BETWEEN 1 AND 5),
	certified_at        TIMESTAMPTZ NOT NULL,
	PRIMARY KEY (reviewer_id, modality)
);
CREATE INDEX IF NOT EXISTS idx_eel_reviewer_spec_modality
	ON eel_reviewer_specialisations (modality, proficiency DESC);

-- Idempotency store for the eel-api-server-rs middleware.
-- One row per accepted (subject_id, idempotency_key) tuple; TTL configured externally.
CREATE TABLE IF NOT EXISTS eel_idempotency_keys (
	key_hash            BYTEA       PRIMARY KEY,        -- SHA-256(subject + endpoint + key)
	subject_id          TEXT        NOT NULL,
	endpoint            TEXT        NOT NULL,
	request_body_hash   BYTEA       NOT NULL,
	response_status     SMALLINT    NOT NULL,
	response_body       BYTEA       NOT NULL,
	response_headers    JSONB       NOT NULL DEFAULT '{}',
	first_seen_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	last_hit_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	hit_count           INT         NOT NULL DEFAULT 1,
	expires_at          TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '24 hours')
);
CREATE INDEX IF NOT EXISTS idx_eel_idempotency_expires
	ON eel_idempotency_keys (expires_at);
