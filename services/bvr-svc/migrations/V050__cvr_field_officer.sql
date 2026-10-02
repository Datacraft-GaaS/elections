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

-- V050: CVR field-officer capture support (WP-4.10, M35 extension)
-- Creates eel_voter_register_events (source of truth for CVR pipeline) and
-- eel_cvr_field_captures (field-officer capture metadata).
--
-- DPA §41 (Data Protection Act 2019) automated-processing gate:
-- Any automated eligibility decision derived from eel_cvr_field_captures
-- MUST be gated by a human reviewer before it is committed to the voter register.
-- The review_verdict column enforces this: only 'accepted' rows may trigger
-- downstream voter-register mutations; 'pending' rows are invisible to the
-- gazette batch. See ADR-0025 for the full DPA §41 compliance note.

-- -----------------------------------------------------------------------
-- eel_voter_register_events
-- -----------------------------------------------------------------------
-- Source values:
--   CRS              Civil Registration Service (batch SFTP, daily)
--   NRB              National Registration Bureau (batch SFTP, weekly)
--   KRA              KRA-iTax dormancy signals (quarterly)
--   FAMILY_SHORTCODE *400# USSD provisional death report
--   OFFICER_MANUAL   Officer triage console (Django, JWT auth)
--   FIELD_OFFICER    Field-officer mobile capture (M35 extension, this migration)

CREATE TABLE IF NOT EXISTS eel_voter_register_events (
    event_id            UUID        NOT NULL PRIMARY KEY,
    voter_id            UUID        NOT NULL,
    event_type          TEXT        NOT NULL
        CHECK (event_type IN (
            'DEATH_PROPOSED', 'DEATH_CONFIRMED', 'ID_CORRECTION',
            'DORMANCY_FLAG', 'FAMILY_NOTIFICATION',
            'REMOVAL_APPROVED', 'REMOVAL_REJECTED', 'REINSTATEMENT'
        )),
    source              TEXT        NOT NULL
        CHECK (source IN (
            'CRS', 'NRB', 'KRA', 'FAMILY_SHORTCODE', 'OFFICER_MANUAL',
            'FIELD_OFFICER'
        )),
    confidence_score    REAL,
    batch_id            UUID,
    officer_id          UUID,
    decision            TEXT        NOT NULL DEFAULT 'PENDING'
        CHECK (decision IN ('APPROVED', 'REJECTED', 'PENDING', 'HELD')),
    decision_basis      TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at         TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_eel_vre_voter
    ON eel_voter_register_events (voter_id);
CREATE INDEX IF NOT EXISTS idx_eel_vre_source_type
    ON eel_voter_register_events (source, event_type);
CREATE INDEX IF NOT EXISTS idx_eel_vre_decision
    ON eel_voter_register_events (decision, created_at DESC)
    WHERE decision = 'PENDING';

-- -----------------------------------------------------------------------
-- eel_cvr_field_captures
-- -----------------------------------------------------------------------
-- Stores per-capture metadata written by the field-officer mobile app.
-- All PII fields (officer SPIFFE ID, captured biometrics) are encrypted
-- at the application layer before storage (eel-crypto-rs KEK, data class A).
--
-- DPA §41 gate (repeated for proximity to the table definition):
-- The review_verdict column must be 'accepted' before any downstream
-- voter-register mutation is permitted. Automated ML-derived verdicts MUST
-- first pass through a human reviewer's PATCH to /reviewer-decision.

CREATE TABLE IF NOT EXISTS eel_cvr_field_captures (
    capture_id          UUID        NOT NULL PRIMARY KEY,
    event_id            UUID        NOT NULL
        REFERENCES eel_voter_register_events (event_id) ON DELETE RESTRICT,

    -- Officer identity
    officer_id          UUID        NOT NULL,
    officer_spiffe      TEXT        NOT NULL,   -- e.g. spiffe://eel.internal/agent/field/KE-007
    kit_id              TEXT        NOT NULL,   -- serial number of the field kit

    -- Capture timing and location
    captured_at         TIMESTAMPTZ NOT NULL,
    geo_lat             DOUBLE PRECISION NOT NULL,
    geo_lon             DOUBLE PRECISION NOT NULL,
    geo_accuracy_m      REAL        NOT NULL,

    -- Certificate artefact
    cert_blob_ref       TEXT        NOT NULL,   -- MinIO URI: minio://eel-cvr-certs/{capture_id}.jpg
    cert_sha256         BYTEA       NOT NULL,   -- server-recomputed; client hash rejected on mismatch
    ocr_confidence      REAL,
    ocr_extracted       JSONB,

    -- Integrity
    officer_hsm_signature BYTEA     NOT NULL,  -- Ed25519 over SHA-256(cert_blob) signed by kit HSM

    -- Offline / sync lifecycle
    offline_queued_at   TIMESTAMPTZ,           -- when the capture was queued on-device
    synced_at           TIMESTAMPTZ,           -- when the backend accepted it

    -- Review workflow (DPA §41: human-in-the-loop gate)
    review_verdict      TEXT        NOT NULL DEFAULT 'pending'
        CHECK (review_verdict IN ('pending', 'accepted', 'rejected', 'manual_review')),
    review_at           TIMESTAMPTZ,
    reviewer_id         UUID
);

-- Lookups by officer (queue management, history screen)
CREATE INDEX IF NOT EXISTS idx_eel_cfc_officer_time
    ON eel_cvr_field_captures (officer_id, captured_at DESC);

-- Adjudicator triage queue
CREATE INDEX IF NOT EXISTS idx_eel_cfc_verdict_pending
    ON eel_cvr_field_captures (review_verdict, captured_at DESC)
    WHERE review_verdict = 'pending';

-- Geographic queries (district-level heatmaps, coverage reporting)
-- Approximated via btree; replace with PostGIS GIST if extension is available.
CREATE INDEX IF NOT EXISTS idx_eel_cfc_geo
    ON eel_cvr_field_captures (geo_lat, geo_lon);

COMMENT ON TABLE eel_cvr_field_captures IS
    'Field-officer certificate captures for CVR (M35 extension). '
    'DPA §41 automated-processing gate: review_verdict must be ''accepted'' '
    'before any voter-register mutation is permitted.';

COMMENT ON COLUMN eel_cvr_field_captures.cert_sha256 IS
    'SHA-256 digest recomputed server-side from cert_blob_ref content. '
    'Client-declared hash is compared at intake; mismatch returns HTTP 422.';

COMMENT ON COLUMN eel_cvr_field_captures.officer_hsm_signature IS
    'Ed25519 signature produced by the field kit''s HSM (TPM 2.0 or SE) over '
    'SHA-256(cert_blob). Verified by cvr/field_notifications.rs at intake.';
