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

-- Cancelable biometrics + FHE bindings + template protection registry.
-- Enables ISO/IEC 24745 conformance (renewability + unlinkability).

CREATE TABLE IF NOT EXISTS eel_bio_protection_schemes (
	scheme_id         TEXT        PRIMARY KEY,      -- 'biohash-v1', 'irreversible-shielding-v2'
	family            TEXT        NOT NULL CHECK (family IN ('biohash','shielding','fuzzy_vault','fuzzy_commit','random_projection','fhe_ckks','fhe_bfv','smpc_spdz')),
	revocable         BOOLEAN     NOT NULL,
	unlinkable        BOOLEAN     NOT NULL,
	irreversible      BOOLEAN     NOT NULL,
	iso_24745_class   TEXT        NOT NULL CHECK (iso_24745_class IN ('cancelable','protected','none')),
	parameter_schema  JSONB       NOT NULL,
	created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	retired_at        TIMESTAMPTZ
);

-- Which template is under which protection scheme with what user-key.
CREATE TABLE IF NOT EXISTS eel_template_protections (
	protection_id     UUID        PRIMARY KEY,
	template_id       UUID        NOT NULL REFERENCES eel_gallery_templates(template_id),
	scheme_id         TEXT        NOT NULL REFERENCES eel_bio_protection_schemes(scheme_id),
	user_key_hash     BYTEA       NOT NULL,          -- SHA-256 of the per-subject secret
	shielded_blob     BYTEA       NOT NULL,
	revocation_ref    UUID,                          -- when non-null, this protection is revoked
	created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	last_used_at      TIMESTAMPTZ
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_template_protection
	ON eel_template_protections (template_id, scheme_id, user_key_hash)
	WHERE revocation_ref IS NULL;

-- FHE contexts: parameter sets shared across a matching cohort.
CREATE TABLE IF NOT EXISTS eel_fhe_contexts (
	context_id        UUID        PRIMARY KEY,
	scheme            TEXT        NOT NULL CHECK (scheme IN ('ckks','bfv','tfhe','bgv')),
	poly_modulus_deg  INT         NOT NULL,
	coeff_moduli_bits INT[]       NOT NULL,
	scale_bits        INT,
	security_level    INT         NOT NULL CHECK (security_level IN (128,192,256)),
	pubkey_blob       BYTEA       NOT NULL,
	relin_key_blob    BYTEA,
	galois_key_blob   BYTEA,
	created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	rotated_at        TIMESTAMPTZ
);

-- SMPC session registry (for threshold-of-N matcher).
CREATE TABLE IF NOT EXISTS eel_smpc_sessions (
	session_id        UUID        PRIMARY KEY,
	protocol          TEXT        NOT NULL CHECK (protocol IN ('spdz','bgw','gmw','yao','abplus')),
	party_count       SMALLINT    NOT NULL CHECK (party_count >= 2),
	threshold         SMALLINT    NOT NULL,
	started_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	completed_at      TIMESTAMPTZ,
	outcome           TEXT        CHECK (outcome IN ('match','no_match','abort','timeout'))
);
