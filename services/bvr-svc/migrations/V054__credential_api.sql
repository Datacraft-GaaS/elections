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

-- Credential API surface — postgres for public contract (no in-memory)
-- Replaces Arc<Mutex<HashMap<Uuid, Credential>>> in credential/http.rs AppState.

CREATE TABLE IF NOT EXISTS credential_api_credentials (
    credential_id   UUID PRIMARY KEY,
    credential_type TEXT NOT NULL,
    holder_id       UUID NOT NULL,
    holder_name     TEXT NOT NULL,
    issuer_id       TEXT NOT NULL,
    issued_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','revoked','expired')),
    claims          JSONB NOT NULL DEFAULT '{}'::jsonb,
    hsm_sig         TEXT NOT NULL DEFAULT '',
    batch_id        UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS credential_api_batches (
    batch_id        UUID PRIMARY KEY,
    credential_type TEXT NOT NULL,
    total_issued    INTEGER NOT NULL DEFAULT 0,
    total_failed    INTEGER NOT NULL DEFAULT 0,
    total_revoked   INTEGER NOT NULL DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'completed',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at    TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_credential_api_credentials_holder ON credential_api_credentials(holder_id);
CREATE INDEX IF NOT EXISTS idx_credential_api_credentials_batch ON credential_api_credentials(batch_id);
CREATE INDEX IF NOT EXISTS idx_credential_api_credentials_status ON credential_api_credentials(status);

-- Reversible: DROP TABLE IF EXISTS credential_api_batches, credential_api_credentials CASCADE;
