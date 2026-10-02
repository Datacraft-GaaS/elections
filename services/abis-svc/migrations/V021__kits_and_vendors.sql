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

-- Kit registry + kit attestation events + vendor router registry.
-- Kit registry is the source of truth for BvrKit/BviKit device identity;
-- attestation events form an append-only audit chain per device.

CREATE TABLE IF NOT EXISTS eel_kits (
	kit_id              TEXT        PRIMARY KEY,
	kit_type            TEXT        NOT NULL CHECK (kit_type IN ('bvr','bvi','rts','legacy_android','diaspora')),
	device_model        TEXT        NOT NULL,
	firmware_version    TEXT        NOT NULL,
	cert_fingerprint    BYTEA       NOT NULL,       -- SHA-256 of client cert
	spiffe_id           TEXT        NOT NULL,       -- spiffe://eel.local/kit/<id>
	attestation_profile TEXT        NOT NULL
	                                CHECK (attestation_profile IN (
	                                    'tdx','sev_snp','play_integrity','app_attest','tpm','software_legacy'
	                                )),
	county_code         TEXT,                       -- home county for cohort routing
	station_id          TEXT,                       -- current polling station assignment
	first_seen_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	last_seen_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	revoked_at          TIMESTAMPTZ,
	revocation_reason   TEXT,
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_kits_cert_fingerprint
	ON eel_kits (cert_fingerprint);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_kits_spiffe_id
	ON eel_kits (spiffe_id);
CREATE INDEX IF NOT EXISTS idx_eel_kits_station
	ON eel_kits (station_id) WHERE revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_eel_kits_last_seen
	ON eel_kits (last_seen_at DESC);

-- Append-only kit attestation events. Ties every enrolment/verify request to
-- a verified TDX/SEV/PlayIntegrity report at that moment.
CREATE TABLE IF NOT EXISTS eel_kit_attestation_events (
	event_id            UUID        PRIMARY KEY,
	kit_id              TEXT        NOT NULL REFERENCES eel_kits(kit_id),
	occurred_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	nonce               BYTEA       NOT NULL,
	report_type         TEXT        NOT NULL,
	report_bytes        BYTEA       NOT NULL,
	report_sha256       BYTEA       NOT NULL,
	verdict             TEXT        NOT NULL CHECK (verdict IN ('accept','reject','warn')),
	policy_version      TEXT        NOT NULL,
	rejected_reason     TEXT,
	prev_event_sha256   BYTEA,                      -- hash chain per kit
	this_event_sha256   BYTEA       NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_eel_kit_attest_kit_time
	ON eel_kit_attestation_events (kit_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_eel_kit_attest_verdict
	ON eel_kit_attestation_events (verdict, occurred_at DESC)
	WHERE verdict <> 'accept';

-- Nonce replay protection: unique per (kit, nonce) over the freshness window.
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_kit_attest_nonce
	ON eel_kit_attestation_events (kit_id, nonce);

-- Vendor router registry: which vendor SDKs the router may dispatch to per modality.
CREATE TABLE IF NOT EXISTS eel_vendor_registry (
	vendor_id           TEXT        PRIMARY KEY,
	vendor_name         TEXT        NOT NULL,
	modalities          TEXT[]      NOT NULL,       -- {fingerprint, face, iris}
	endpoint_url        TEXT        NOT NULL,
	transport           TEXT        NOT NULL CHECK (transport IN ('http','grpc','process')),
	priority            SMALLINT    NOT NULL DEFAULT 100,
	weight              SMALLINT    NOT NULL DEFAULT 100,
	capabilities        JSONB       NOT NULL DEFAULT '{}',
	health_status       TEXT        NOT NULL DEFAULT 'unknown'
	                                CHECK (health_status IN ('healthy','degraded','unhealthy','unknown')),
	last_health_check   TIMESTAMPTZ,
	error_rate_15m      REAL        NOT NULL DEFAULT 0.0,
	latency_p99_ms      INT,
	circuit_state       TEXT        NOT NULL DEFAULT 'closed'
	                                CHECK (circuit_state IN ('closed','open','half_open')),
	enabled             BOOLEAN     NOT NULL DEFAULT TRUE,
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_eel_vendor_registry_health
	ON eel_vendor_registry (health_status, priority)
	WHERE enabled = TRUE;
