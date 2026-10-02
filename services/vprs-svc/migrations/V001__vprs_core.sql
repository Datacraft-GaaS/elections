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

-- VPRS core schema (WP-4.10, M04).
-- Tables referenced by services/vprs-svc/crates/vprs-api runtime queries.

CREATE TABLE IF NOT EXISTS vprs_applications (
    app_id            TEXT PRIMARY KEY,
    id_number_hash    TEXT NOT NULL UNIQUE,
    status            TEXT NOT NULL DEFAULT 'draft',
    biographic_fields JSONB NOT NULL DEFAULT '{}'::jsonb,
    consent_tick      BOOLEAN NOT NULL DEFAULT FALSE,
    qr_token_hash     TEXT UNIQUE,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at        TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS vprs_otp_sessions (
    session_id    TEXT PRIMARY KEY,
    app_id        TEXT NOT NULL UNIQUE REFERENCES vprs_applications(app_id),
    code_hash     TEXT NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    verified      BOOLEAN NOT NULL DEFAULT FALSE,
    locked_until  TIMESTAMPTZ,
    expires_at    TIMESTAMPTZ NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
