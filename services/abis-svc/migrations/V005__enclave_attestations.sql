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

-- Enclave attestation records: TDX/SEV-SNP or software profile.
-- Data class C per ADR-0016.
CREATE TABLE IF NOT EXISTS eel_enclave_attestations (
    enclave_id       TEXT        PRIMARY KEY,
    worker_id        TEXT        NOT NULL,
    profile          TEXT        NOT NULL CHECK (profile IN ('software','tdx','sev_snp')),
    attestation_hash TEXT        NOT NULL,
    batch_id         TEXT,
    verified_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_eel_enclave_attestations_worker ON eel_enclave_attestations(worker_id);
CREATE INDEX IF NOT EXISTS idx_eel_enclave_attestations_batch  ON eel_enclave_attestations(batch_id);
